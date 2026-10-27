"""Owner-maintained inventory and cleanup entry point. Uses existing gh authentication."""
import argparse
import concurrent.futures
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import urllib.parse


def command(args, cwd=None, timeout=45, optional=False):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    if len(result.stdout.encode()) > 8 * 1024 * 1024:
        raise ValueError('Command output exceeds budget')
    if result.returncode and not optional:
        raise RuntimeError(f'{args[0]} failed ({result.returncode}): {result.stderr[:500]}')
    return result


def api(path, method='GET', body=None):
    args = ['gh', 'api', path, '--method', method]
    if body is not None:
        args += ['--input', '-']
    result = subprocess.run(args, input=json.dumps(body) if body is not None else None,
                            capture_output=True, text=True, timeout=45)
    if len(result.stdout.encode()) > 8 * 1024 * 1024:
        raise ValueError('API response exceeds budget')
    try:
        value = json.loads(result.stdout)
    except json.JSONDecodeError:
        return {'state': 'unavailable', 'error': result.stderr[:500]}
    if result.returncode:
        return {'state': 'denied' if str(value.get('status')) in ['401', '403'] else 'unavailable',
                'status': value.get('status'), 'message': value.get('message', '')[:500]}
    return {'state': 'observed', 'data': value}


def pages(path):
    values = []
    for page in range(1, 21):
        result = api(path + ('&' if '?' in path else '?') + f'per_page=100&page={page}')
        if result['state'] != 'observed':
            return {**result, 'partial': values}
        part = result['data']
        if not isinstance(part, list):
            raise ValueError('Expected paginated array')
        values += part
        if len(part) < 100:
            return {'state': 'observed', 'data': values}
    return {'state': 'partial', 'data': values, 'message': 'Pagination budget reached'}


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    temporary = path.with_suffix(path.suffix + '.tmp')
    with temporary.open('w', encoding='utf8') as file:
        os.chmod(temporary, 0o600)
        json.dump(value, file, ensure_ascii=False, indent=2)
        file.write('\n')
    temporary.replace(path)


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def owner_repository(value):
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', value):
        raise ValueError('Explicit owner/repository required')
    return value


def inventory(args):
    workspace = args.workspace.resolve()
    output = args.output.resolve()
    if not args.owner or len(args.owner) != len(set(args.owner)):
        raise ValueError('Explicit unique managed owners required')
    identity = api('user')
    if identity['state'] != 'observed':
        raise ValueError('Authenticated GitHub identity unavailable')
    repositories = pages('user/repos?affiliation=owner,collaborator,organization_member&sort=full_name')
    if repositories['state'] != 'observed':
        raise ValueError('Repository discovery incomplete')
    selected = [r for r in repositories['data'] if r['owner']['login'] in args.owner]
    if len({r['id'] for r in selected}) != len(selected):
        raise ValueError('Duplicate repository identity')
    write(output / 'scope.json', {'observedAt': now(), 'actor': identity['data']['login'],
                                 'owners': args.owner, 'repositories': selected})

    def observe(meta):
        name = owner_repository(meta['full_name'])
        root = workspace / 'organizations' / name
        default = meta['default_branch']
        prefix = 'repos/' + name
        result = {'repository': name, 'observedAt': now(), 'metadata': meta,
                  'defaultBranch': api(prefix + '/branches/' + urllib.parse.quote(default, safe='')),
                  'classicProtection': api(prefix + '/branches/' + urllib.parse.quote(default, safe='') + '/protection'),
                  'rulesets': pages(prefix + '/rulesets?includes_parents=true'),
                  'effectiveRules': pages(prefix + '/rules/branches/' + urllib.parse.quote(default, safe='')),
                  'issues': pages(prefix + '/issues?state=open'),
                  'pulls': pages(prefix + '/pulls?state=open'),
                  'branches': pages(prefix + '/branches'),
                  'local': {'exists': (root / '.git').exists()}}
        if result['rulesets']['state'] == 'observed':
            result['rulesetDetails'] = [api(prefix + '/rulesets/' + str(v['id']))
                                       for v in result['rulesets']['data'][:12]]
        if result['local']['exists']:
            git = lambda *words: command(['git', *words], root).stdout.strip()
            result['local'].update({'head': git('rev-parse', 'HEAD'),
                'branch': git('branch', '--show-current'),
                'status': git('status', '--porcelain=v1'),
                'origin': git('remote', 'get-url', 'origin'),
                'branches': git('for-each-ref', '--format=%(refname:short)\t%(objectname)\t%(upstream:short)', 'refs/heads'),
                'worktrees': git('worktree', 'list', '--porcelain')})
            for filename in ['README.md', 'Cargo.toml', 'package.json', 'repository.toml']:
                path = root / filename
                if path.is_file() and not path.is_symlink() and path.stat().st_size < 256 * 1024:
                    result['local'][filename] = path.read_text()[:20000]
        write(output / 'repositories' / (name.replace('/', '__') + '.json'), result)
        count = lambda key: len(result[key].get('data', [])) if result[key]['state'] == 'observed' else None
        summary = {'repository': name, 'private': meta['private'], 'description': meta['description'],
                   'default': default, 'protected': result['defaultBranch'].get('data', {}).get('protected'),
                   'protectionState': result['classicProtection']['state'], 'rulesets': count('rulesets'),
                   'openIssuesAndPRs': count('issues'), 'openPRs': count('pulls'), 'branches': count('branches'),
                   'localDirty': bool(result['local'].get('status')), 'admin': meta.get('permissions', {}).get('admin', False)}
        print(json.dumps(summary), flush=True)
        return summary

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        futures = {pool.submit(observe, m): m['full_name'] for m in selected}
        results = []
        for future in concurrent.futures.as_completed(futures):
            try:
                results.append(future.result())
            except Exception as error:
                failure = {'repository': futures[future], 'state': 'failed', 'error': str(error)}
                results.append(failure)
                print(json.dumps(failure), flush=True)
            write(output / 'inventory.json', sorted(results, key=lambda r: r['repository']))
    write(output / 'complete.json', {'completedAt': now(), 'repositories': len(selected),
        'observed': len(results), 'failures': sum(r.get('state') == 'failed' for r in results)})


def pull_eligibility(pull):
    """Produce reasons, never confuse a passing subset with completed CI."""
    reasons = []
    if pull.get('state', 'OPEN') != 'OPEN':
        reasons.append('not-open')
    if pull.get('isDraft'):
        reasons.append('draft')
    if pull.get('mergeable') != 'MERGEABLE':
        reasons.append('not-confirmed-mergeable')
    if pull.get('mergeStateStatus') not in ['CLEAN', 'HAS_HOOKS']:
        reasons.append('merge-state-' + str(pull.get('mergeStateStatus')))
    if pull.get('reviewDecision') == 'REVIEW_REQUIRED' or pull.get('reviewDecision') == 'CHANGES_REQUESTED':
        reasons.append('review-required')
    checks = pull.get('statusCheckRollup') or []
    if not checks:
        reasons.append('no-CI-evidence')
    for check in checks:
        if check.get('__typename') == 'StatusContext':
            passed = check.get('state') == 'SUCCESS'
        else:
            passed = check.get('status') == 'COMPLETED' and check.get('conclusion') in ['SUCCESS', 'SKIPPED', 'NEUTRAL']
        if not passed:
            reasons.append('CI:' + str(check.get('name', check.get('context'))))
    # A workflow consisting entirely of skipped checks is not functional evidence.
    if checks and not any(c.get('conclusion') == 'SUCCESS' or c.get('state') == 'SUCCESS' for c in checks):
        reasons.append('no-successful-check')
    if not re.fullmatch('[a-f0-9]{40}', pull.get('headRefOid', '')):
        reasons.append('invalid-head')
    return {'eligible': not reasons, 'reasons': reasons}


def report(args):
    root = args.inventory.resolve()
    scope = json.loads((root / 'scope.json').read_text())
    expected = {r['full_name']: r['id'] for r in scope['repositories']}
    records = [json.loads(p.read_text()) for p in sorted((root / 'repositories').glob('*.json'))]
    observed = {r['repository']: r['metadata']['id'] for r in records}
    if len(observed) != len(records) or observed != expected:
        raise ValueError('Incomplete or unexpected repository inventory')
    write(root / 'complete.json', {'completedAt': now(), 'repositories': len(expected),
                                 'observed': len(observed), 'failures': 0})
    issues = [{'repository': r['repository'], **i} for r in records
              for i in r['issues'].get('data', []) if 'pull_request' not in i]
    result = {'repositories': len(records),
        'unprotected': [r['repository'] for r in records if r['defaultBranch'].get('data', {}).get('protected') is False],
        'unobservedProtection': [r['repository'] for r in records if r['defaultBranch']['state'] != 'observed'],
        'blankDescriptions': [r['repository'] for r in records if not r['metadata']['description']],
        'dirtyLocal': [r['repository'] for r in records if r['local'].get('status')],
        'openIssues': len(issues), 'openPulls': sum(len(r['pulls'].get('data', [])) for r in records),
        'remoteBranches': sum(len(r['branches'].get('data', [])) for r in records)}
    write(root / 'issues.json', issues)
    write(root / 'report.json', result)
    print(json.dumps(result, ensure_ascii=False, indent=2))


def pulls(args):
    source = args.inventory.resolve()
    completed = json.loads((source / 'complete.json').read_text())
    if completed['failures'] or completed['observed'] != completed['repositories']:
        raise ValueError('Incomplete inventory; no fleet completeness claim')
    records = []
    for path in sorted((source / 'repositories').glob('*.json')):
        record = json.loads(path.read_text())
        if record['pulls']['state'] != 'observed':
            continue
        for pull in record['pulls']['data']:
            records.append((record['repository'], pull['number']))

    def inspect(item):
        repo, number = item
        fields = 'number,state,title,body,author,labels,isDraft,headRefName,headRefOid,baseRefName,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,files,changedFiles,url'
        result = command(['gh', 'pr', 'view', str(number), '--repo', repo, '--json', fields])
        pull = json.loads(result.stdout)
        pull['repository'] = repo
        pull['admission'] = pull_eligibility(pull)
        if args.patch:
            patch = command(['gh', 'pr', 'diff', str(number), '--repo', repo]).stdout
            patch_path = args.output.resolve() / 'patches' / f"{repo.replace('/', '__')}__{number}.patch"
            patch_path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            patch_path.write_text(patch)
            os.chmod(patch_path, 0o600)
            pull['patchSha256'] = hashlib.sha256(patch.encode()).hexdigest()
        write(args.output.resolve() / 'pulls' / f"{repo.replace('/', '__')}__{number}.json", pull)
        print(json.dumps({'repository': repo, 'number': number, 'title': pull['title'],
                          'head': pull['headRefOid'], **pull['admission']}), flush=True)
        return pull

    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(inspect, records))
    write(args.output.resolve() / 'pulls.json', results)


def check_pull(args):
    repo = owner_repository(args.repository)
    fields = 'state,isDraft,headRefOid,baseRefName,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup'
    pull = json.loads(command(['gh', 'pr', 'view', str(args.number), '--repo', repo, '--json', fields]).stdout)
    admitted = pull_eligibility(pull)
    if pull['headRefOid'] != args.expected_head:
        admitted['eligible'] = False
        admitted['reasons'].append('head-changed-since-review')
    if pull['baseRefName'] != 'main':
        admitted['eligible'] = False
        admitted['reasons'].append('different-base')
    print(json.dumps({'repository': repo, 'number': args.number, 'head': pull['headRefOid'], **admitted}))


def apply(args):
    """Apply a concrete operator-reviewed plan once; preserve uncertain writes."""
    source = args.plan.resolve()
    if args.plan.is_symlink() or source.stat().st_mode & 0o022 or source.stat().st_size > 4 * 1024 * 1024:
        raise ValueError('Private regular bounded plan required')
    plan = json.loads(source.read_text())
    if plan.get('schema') != 'zix-github-maintenance-plan/v1':
        raise ValueError('Unsupported maintenance plan')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True, mode=0o700)
    for index, op in enumerate(plan['operations']):
        repo = owner_repository(op['repository'])
        path = output / f'{index:04d}-{repo.replace("/", "__")}.json'
        if path.exists():
            raise ValueError('Prior operation receipt exists; inspect, do not replay')
        meta = api('repos/' + repo)
        if meta['state'] != 'observed' or meta['data']['id'] != op['repositoryId'] or not meta['data'].get('permissions', {}).get('admin') or meta['data']['archived']:
            raise ValueError('Repository identity/admin authority changed')
        record = {'repository': repo, 'operation': op['kind'], 'planSha256': digest(plan), 'startedAt': now(), 'state': 'prewrite'}
        write(path, record)
        try:
            if op['kind'] == 'metadata':
                if set(op['desired']) - {'description', 'homepage', 'delete_branch_on_merge'} or not op['desired']:
                    raise ValueError('Unsupported metadata write')
                if {k: meta['data'].get(k) for k in op['desired']} != op['expected']:
                    raise ValueError('Metadata drift')
                record['state'] = 'write-starting; do not replay'
                write(path, record)
                result = api('repos/' + repo, 'PATCH', op['desired'])
                if result['state'] != 'observed':
                    raise ValueError('Metadata write not acknowledged: ' + str(result))
                after = api('repos/' + repo)
                if after['state'] != 'observed' or any(after['data'].get(k) != v for k, v in op['desired'].items()):
                    raise ValueError('Metadata readback differs')
            elif op['kind'] == 'rulesets-add':
                if meta['data']['private'] or meta['data']['default_branch'] != 'main':
                    raise ValueError('Only explicitly selected public main rulesets supported')
                current = pages('repos/' + repo + '/rulesets?includes_parents=true')
                if current['state'] != 'observed' or digest(current['data']) != op['expectedRulesetsSha256']:
                    raise ValueError('Ruleset drift or unavailable observation')
                existing = {r['name'] for r in current['data']}
                for wanted in op['desired']:
                    if wanted['name'] in existing or wanted['target'] != 'branch' or wanted['enforcement'] != 'active' or wanted['conditions'] != {'ref_name': {'include': ['refs/heads/main'], 'exclude': []}}:
                        raise ValueError('Ruleset is not an absent active main rule')
                    record['state'] = 'write-starting; do not replay'
                    record['creating'] = wanted['name']
                    write(path, record)
                    result = api('repos/' + repo + '/rulesets', 'POST', wanted)
                    if result['state'] != 'observed':
                        raise ValueError('Ruleset write not acknowledged: ' + str(result))
                    record.setdefault('created', []).append(result['data']['id'])
                    write(path, record)
                    existing.add(wanted['name'])
                branch = api('repos/' + repo + '/branches/main')
                effective = pages('repos/' + repo + '/rules/branches/main')
                if branch['state'] != 'observed' or not branch['data']['protected'] or effective['state'] != 'observed':
                    raise ValueError('Effective protection unavailable')
                required = {r['type'] for w in op['desired'] for r in w['rules']}
                if not required <= {r['type'] for r in effective['data']}:
                    raise ValueError('Effective rules missing')
                record['effectiveRules'] = effective['data']
            elif op['kind'] == 'issue-close':
                issue_path = 'repos/' + repo + '/issues/' + str(op['number'])
                current = api(issue_path)
                if current['state'] != 'observed' or current['data']['updated_at'] != op['expectedUpdatedAt'] or current['data']['state'] != 'open' or 'pull_request' in current['data']:
                    raise ValueError('Issue drift, closed issue or PR')
                if not op.get('evidence') or op['reason'] not in ['completed', 'not_planned']:
                    raise ValueError('Explicit evidence and disposition required')
                body = (current['data'].get('body') or '') + '\n\nMaintenance resolution: ' + op['evidence']
                record['state'] = 'write-starting; do not replay'
                write(path, record)
                result = api(issue_path, 'PATCH', {'state': 'closed', 'state_reason': op['reason'], 'body': body})
                if result['state'] != 'observed' or result['data']['state'] != 'closed':
                    raise ValueError('Issue close not acknowledged')
                after = api(issue_path)
                if after['state'] != 'observed' or after['data']['state'] != 'closed' or after['data'].get('state_reason') != op['reason']:
                    raise ValueError('Issue readback differs')
            else:
                raise ValueError('Unknown maintenance operation')
            record.update(state='verified', completedAt=now())
            write(path, record)
            print(repo, op['kind'], 'verified', flush=True)
        except Exception as error:
            record.update(error=str(error), stoppedAt=now())
            write(path, record)
            print(repo, op['kind'], 'STOPPED', str(error), flush=True)
            raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    inv = sub.add_parser('inventory', help='Observe all accessible repositories in explicit managed owners')
    inv.add_argument('--workspace', type=Path, required=True)
    inv.add_argument('--output', type=Path, required=True)
    inv.add_argument('--owner', action='append', required=True)
    pr = sub.add_parser('pulls', help='Read exact PR heads, diffs and CI; emit conservative merge eligibility')
    pr.add_argument('--inventory', type=Path, required=True)
    pr.add_argument('--output', type=Path, required=True)
    pr.add_argument('--patch', action='store_true', help='Also retain bounded actual PR diffs for operator review')
    summary = sub.add_parser('report', help='Check full scope coverage and project current inventory findings')
    summary.add_argument('--inventory', type=Path, required=True)
    mutation = sub.add_parser('apply', help='Apply a concrete explicit plan once with fresh-state/readback guards')
    mutation.add_argument('--plan', type=Path, required=True)
    mutation.add_argument('--output', type=Path, required=True)
    check = sub.add_parser('check-pull', help='Refresh merge eligibility for the exact operator-reviewed head')
    check.add_argument('--repository', required=True)
    check.add_argument('--number', type=int, required=True)
    check.add_argument('--expected-head', required=True)
    branches = sub.add_parser('branches', help='Fetch current refs and clean only proven merged branches')
    branches.add_argument('--inventory', type=Path, required=True)
    branches.add_argument('--workspace', type=Path, required=True)
    branches.add_argument('--output', type=Path, required=True)
    branches.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    if args.command == 'inventory':
        inventory(args)
    elif args.command == 'pulls':
        pulls(args)
    elif args.command == 'apply':
        apply(args)
    elif args.command == 'report':
        report(args)
    elif args.command == 'check-pull':
        check_pull(args)
    elif args.command == 'branches':
        from .branches import cleanup
        cleanup(args)


if __name__ == '__main__':
    main()
