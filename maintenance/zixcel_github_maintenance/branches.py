"""Merged-branch cleanup with fresh Git refs, open-PR and exact-ref guards."""
import json
from pathlib import Path
import urllib.parse
from .__main__ import api, command, now, owner_repository, pages, write


def protected_names(default):
    return {default, 'main', 'master', 'develop', 'development'}


def merged_pull_proof(pulls, repo, branch, head, default):
    return next((p for p in pulls if p.get('merged_at') and p.get('base', {}).get('ref') == default
        and p.get('head', {}).get('ref') == branch and p['head'].get('sha') == head
        and (p['head'].get('repo') or {}).get('full_name') == repo), None)


def cleanup(args):
    inventory = args.inventory.resolve()
    complete = json.loads((inventory / 'complete.json').read_text())
    if complete['failures'] or complete['observed'] != complete['repositories']:
        raise ValueError('Complete explicit scope required')
    scopes = json.loads((inventory / 'scope.json').read_text())['repositories']
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False, mode=0o700)
    results = []
    for original in scopes:
        repo = owner_repository(original['full_name'])
        root = args.workspace.resolve() / 'organizations' / repo
        record = {'repository': repo, 'observedAt': now(), 'applied': args.apply,
                  'localDeleted': [], 'remoteDeleted': [], 'retained': []}
        destination = output / (repo.replace('/', '__') + '.json')
        write(destination, record)
        try:
            meta = api('repos/' + repo)
            if meta['state'] != 'observed' or meta['data']['id'] != original['id'] or not meta['data'].get('permissions', {}).get('admin') or meta['data']['archived']:
                raise ValueError('Repository identity or authority changed')
            if not (root / '.git').exists():
                raise ValueError('Local checkout absent')
            git = lambda *words, optional=False: command(['git', *words], root, timeout=90, optional=optional)
            origin = git('remote', 'get-url', 'origin').stdout.strip()
            if origin not in [f'https://github.com/{repo}.git', f'https://github.com/{repo}', f'git@github.com:{repo}.git']:
                raise ValueError('Origin is not the selected GitHub repository')
            git('fetch', '--prune', 'origin')
            default = meta['data']['default_branch']
            main = git('rev-parse', 'refs/remotes/origin/' + default).stdout.strip()
            current = git('branch', '--show-current').stdout.strip()
            dirty = bool(git('status', '--porcelain=v1').stdout.strip())
            worktrees = git('worktree', 'list', '--porcelain').stdout
            occupied = {line[7:] for line in worktrees.splitlines() if line.startswith('branch ')}
            local_default = git('rev-parse', '--verify', 'refs/heads/' + default, optional=True)
            if args.apply and local_default.returncode == 0:
                old = local_default.stdout.strip()
                ancestor = git('merge-base', '--is-ancestor', old, main, optional=True).returncode == 0
                if old != main and ancestor:
                    if current == default and not dirty:
                        git('merge', '--ff-only', 'origin/' + default)
                        record['defaultFastForwarded'] = {'from': old, 'to': main}
                    elif 'refs/heads/' + default not in occupied:
                        git('update-ref', 'refs/heads/' + default, main, old)
                        record['defaultFastForwarded'] = {'from': old, 'to': main}
                elif old != main:
                    record['retained'].append({'branch': default, 'reason': 'local-default-has-unmerged-commits'})
            open_pulls = pages('repos/' + repo + '/pulls?state=open')
            remote = pages('repos/' + repo + '/branches')
            if open_pulls['state'] != 'observed' or remote['state'] != 'observed':
                raise ValueError('Open-PR or branch observation incomplete')
            open_heads = {p['head']['ref'] for p in open_pulls['data']
                          if (p['head'].get('repo') or {}).get('full_name') == repo}
            remote_heads = {b['name']: b for b in remote['data']}
            local_heads = {}
            for line in git('for-each-ref', '--format=%(refname:short)\t%(objectname)', 'refs/heads').stdout.splitlines():
                name, head = line.split('\t'); local_heads[name] = head
            candidates = sorted((set(local_heads) | set(remote_heads)) - protected_names(default))
            for branch in candidates:
                if branch in open_heads:
                    record['retained'].append({'branch': branch, 'reason': 'open-PR'})
                    continue
                if remote_heads.get(branch, {}).get('protected'):
                    record['retained'].append({'branch': branch, 'reason': 'protected-branch'})
                    continue
                if 'refs/heads/' + branch in occupied and (current != branch or dirty):
                    record['retained'].append({'branch': branch, 'reason': 'dirty-or-other-active-worktree'})
                    continue
                heads = {h for h in [local_heads.get(branch), remote_heads.get(branch, {}).get('commit', {}).get('sha')] if h}
                proofs = {}
                closed = None
                for head in heads:
                    if git('merge-base', '--is-ancestor', head, main, optional=True).returncode == 0:
                        proofs[head] = {'kind': 'ancestor-of-default', 'defaultHead': main}
                        continue
                    if closed is None:
                        query = urllib.parse.urlencode({'state': 'closed', 'head': repo.split('/')[0] + ':' + branch})
                        closed = pages('repos/' + repo + '/pulls?' + query)
                    if closed['state'] == 'observed':
                        proof = merged_pull_proof(closed['data'], repo, branch, head, default)
                        if proof and git('merge-base', '--is-ancestor', proof['merge_commit_sha'], main, optional=True).returncode == 0:
                            proofs[head] = {'kind': 'exact-head-merged-PR', 'pull': proof['number'],
                                            'mergeCommit': proof['merge_commit_sha'], 'defaultHead': main}
                record.setdefault('proposals', []).append({'branch': branch, 'proofs': proofs})
                write(destination, record)
                remote_head = remote_heads.get(branch, {}).get('commit', {}).get('sha')
                if remote_head in proofs and args.apply:
                    # A lease rejects a branch advanced after this proof. Never force-push a ref.
                    fresh_pulls = pages('repos/' + repo + '/pulls?state=open&head=' + urllib.parse.quote(repo.split('/')[0] + ':' + branch, safe=''))
                    if fresh_pulls['state'] != 'observed' or fresh_pulls['data']:
                        record['retained'].append({'branch': branch, 'reason': 'open-PR-state-changed'})
                        continue
                    record['writePending'] = {'branch': branch, 'head': remote_head, 'operation': 'delete-remote'}
                    write(destination, record)
                    git('push', '--force-with-lease=refs/heads/' + branch + ':' + remote_head, 'origin', ':refs/heads/' + branch)
                    if git('ls-remote', '--heads', 'origin', 'refs/heads/' + branch).stdout.strip():
                        raise ValueError('Remote deletion readback differs; do not replay')
                    record['remoteDeleted'].append({'branch': branch, 'head': remote_head, 'proof': proofs[remote_head]})
                    record.pop('writePending', None)
                    write(destination, record)
                local_head = local_heads.get(branch)
                if local_head not in proofs:
                    if local_head or remote_head not in proofs:
                        record['retained'].append({'branch': branch, 'reason': 'unmerged-or-unproven-changes'})
                    continue
                if args.apply:
                    if current == branch and not dirty:
                        if 'refs/heads/' + default in occupied and default != current:
                            record['retained'].append({'branch': branch, 'reason': 'default-in-another-worktree'})
                            continue
                        git('switch', default); current = default
                        occupied.discard('refs/heads/' + branch)
                    if 'refs/heads/' + branch in occupied:
                        record['retained'].append({'branch': branch, 'reason': 'dirty-or-active-worktree'})
                        continue
                    git('update-ref', '-d', 'refs/heads/' + branch, local_head)
                    if git('show-ref', '--verify', '--quiet', 'refs/heads/' + branch, optional=True).returncode == 0:
                        raise ValueError('Local deletion readback differs')
                    record['localDeleted'].append({'branch': branch, 'head': local_head, 'proof': proofs[local_head]})
                    write(destination, record)
            record.update(state='complete', completedAt=now(), dirtySourcePreserved=dirty)
        except Exception as error:
            record.update(state='stopped', error=str(error))
        write(destination, record)
        results.append(record)
        write(output / 'summary.json', results)
        print(repo, record['state'], 'local', len(record['localDeleted']), 'remote', len(record['remoteDeleted']), record.get('error', ''), flush=True)
