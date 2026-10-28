"""Run an operator-reviewed local CI plan against an exact integration tree.

This executes explicit commands, never discovers or trusts commands from PR text,
and does not grant merge authority or manufacture GitHub check results.
"""
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import re

from .__main__ import api, command, digest, now, owner_repository, write


def prepare(args):
    """Refresh a reviewed plan without trusting a PR's comparison base SHA."""
    plan = json.loads(args.plan.read_text())
    if plan['schema'] != 'zix-local-ci-plan/v1' or plan.get('verifyMerged'):
        raise ValueError('Open-PR local CI plan required')
    if args.output.exists():
        raise ValueError('Fresh plan output required')
    owner_repository(plan['repository'])
    cwd = Path(plan['checkout']).resolve(strict=True)
    if command(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=cwd).stdout.strip():
        raise ValueError('Preparation must not overwrite tracked changes')
    pr = api(f"repos/{plan['repository']}/pulls/{plan['number']}")
    base = api(f"repos/{plan['repository']}/git/ref/heads/main")
    if (pr['state'] != 'observed' or base['state'] != 'observed'
            or pr['data']['state'] != 'open' or pr['data']['base']['ref'] != 'main'
            or pr['data']['head']['sha'] != plan['head']):
        raise ValueError('Unavailable or changed PR head/main')
    origin = command(['git', 'remote', 'get-url', 'origin'], cwd=cwd).stdout.strip()
    allowed = [f"https://github.com/{plan['repository']}", f"git@github.com:{plan['repository']}"]
    if origin.removesuffix('.git') not in allowed:
        raise ValueError('Checkout origin differs from explicit repository')
    command(['git', 'fetch', 'origin', 'main', f"refs/pull/{plan['number']}/head"], cwd=cwd, timeout=120)
    plan['base'] = base['data']['object']['sha']
    tree = command(['git', 'merge-tree', '--write-tree', plan['base'], plan['head']], cwd=cwd).stdout.splitlines()[0]
    if not re.fullmatch('[0-9a-f]{40}', tree):
        raise ValueError('No conflict-free integration tree')
    for check in plan['checks']:
        if check.get('workflowSha256'):
            file = check['name'].split('/')[0]
            source = command(['git', 'show', tree + ':.github/workflows/' + file], cwd=cwd).stdout
            if hashlib.sha256(source.encode()).hexdigest() != check['workflowSha256']:
                raise ValueError('Selected workflow changed; review commands again')
    commit = command(['git', '-c', 'user.name=Local CI preparation', '-c',
                      'user.email=local-ci@localhost', 'commit-tree', tree,
                      '-p', plan['base'], '-p', plan['head'], '-m',
                      'Local CI integration; not a published commit'], cwd=cwd).stdout.strip()
    command(['git', 'switch', '--detach', commit], cwd=cwd)
    plan['tree'] = tree
    previous = plan.get('env', {}).get('RUNNER_TEMP')
    if previous:
        runtime = args.output.resolve().with_suffix('').with_name(args.output.stem + '-runtime')
        runtime.mkdir(mode=0o700, parents=True, exist_ok=False)
        plan = json.loads(json.dumps(plan).replace(previous, str(runtime)))
        for name in ['GITHUB_ENV', 'GITHUB_OUTPUT', 'GITHUB_STEP_SUMMARY']:
            if plan['env'].get(name):
                Path(plan['env'][name]).touch(mode=0o600, exist_ok=False)
    plan.setdefault('env', {})['GITHUB_SHA'] = commit
    observe(plan)
    write(args.output, plan)
    print(plan['repository'], plan['number'], 'prepared', tree, flush=True)


def observe(plan):
    owner_repository(plan['repository'])
    result = api(f"repos/{plan['repository']}/pulls/{plan['number']}")
    if result['state'] != 'observed':
        raise ValueError('PR observation unavailable')
    pr = result['data']
    state_matches = (pr['state'] == 'closed' and pr.get('merged')) if plan.get('verifyMerged') else pr['state'] == 'open'
    if not state_matches or pr['head']['sha'] != plan['head'] or pr['base']['ref'] != 'main':
        raise ValueError('PR head/branch drift or unexpected PR state')
    # A PR's base.sha can describe its older comparison snapshot. Only the
    # actual branch reference proves which main integration is being tested.
    base = api(f"repos/{plan['repository']}/git/ref/heads/main")
    if base['state'] != 'observed' or base['data']['object']['sha'] != plan['base']:
        raise ValueError('Current main drift or unavailable main reference')
    return pr


def execute(argv, cwd, env, log, timeout):
    if not argv or not all(isinstance(x, str) for x in argv):
        raise ValueError('An explicit argument vector is required')
    with log.open('xb') as output:
        os.chmod(log, 0o600)
        process = subprocess.Popen(argv, cwd=cwd, env=env, stdout=output,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        try:
            deadline = time.monotonic() + timeout
            while True:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise subprocess.TimeoutExpired(argv, timeout)
                if log.stat().st_size > 64 * 1024 * 1024:
                    raise ValueError('Local check log exceeds 64 MiB budget')
                try:
                    return process.wait(timeout=min(0.25, remaining))
                except subprocess.TimeoutExpired:
                    continue
        except BaseException:
            # Only the process group created by this invocation is terminated.
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            raise


def run(args):
    plan = json.loads(args.plan.read_text())
    if plan['schema'] != 'zix-local-ci-plan/v1' or not plan.get('checks'):
        raise ValueError('Explicit nonempty local CI plan required')
    observe(plan)
    cwd = Path(plan['checkout']).resolve(strict=True)
    tree = command(['git', 'rev-parse', 'HEAD^{tree}'], cwd=cwd).stdout.strip()
    if tree != plan['tree']:
        raise ValueError('Integration tree differs from reviewed plan')
    for sha in (plan['base'], plan['head']):
        command(['git', 'merge-base', '--is-ancestor', sha, 'HEAD'], cwd=cwd)
    if command(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=cwd).stdout.strip():
        raise ValueError('Tracked changes in validation checkout')
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    record = dict(schema='zix-local-ci-evidence/v1', plan=plan,
                  planSha256=digest(plan), startedAt=now(), state='running', checks=[])
    receipt = args.output / 'receipt.json'
    write(receipt, record)
    env = os.environ.copy()
    env.update(plan.get('env', {}))
    try:
        for index, check in enumerate(plan['checks']):
            timeout = check['timeoutSeconds']
            if not isinstance(timeout, int) or not 1 <= timeout <= 3600:
                raise ValueError('Check timeout must be 1..3600 seconds')
            log = args.output / f'{index:03}.log'
            print(plan['repository'], plan['number'], check['name'], flush=True)
            result = execute(check['argv'], cwd, env, log, timeout)
            record['checks'].append(dict(name=check['name'], exitCode=result,
                                        logSha256=hashlib.sha256(log.read_bytes()).hexdigest()))
            write(receipt, record)
            if result:
                raise ValueError('Local check failed: ' + check['name'])
        if command(['git', 'diff', 'HEAD', '--exit-code'], cwd=cwd).stdout.strip():
            raise ValueError('Validation modified tracked source')
        observe(plan)
        record['state'] = 'passed'
    except BaseException as error:
        record.update(state='failed', error=str(error))
        raise
    finally:
        record['completedAt'] = now()
        write(receipt, record)


def merge(args):
    """Normal GitHub PR merge after explicit review of complete local evidence."""
    record = json.loads(args.evidence.read_text())
    if record.get('schema') != 'zix-local-ci-evidence/v1' or record.get('state') != 'passed':
        raise ValueError('Passed local CI evidence required')
    plan = record['plan']
    if plan.get('verifyMerged'):
        raise ValueError('Post-merge verification evidence cannot authorize another merge')
    if record['planSha256'] != digest(plan) or len(record['checks']) != len(plan['checks']):
        raise ValueError('Changed or incomplete local evidence')
    for index, check in enumerate(record['checks']):
        log = args.evidence.parent / f'{index:03}.log'
        if check['exitCode'] != 0 or hashlib.sha256(log.read_bytes()).hexdigest() != check['logSha256']:
            raise ValueError('Failed or changed local check log')
    pr = observe(plan)
    if pr['draft']:
        raise ValueError('Draft requires an explicit completion review first')
    args.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    receipt = args.output / 'merge.json'
    value = dict(repository=plan['repository'], number=plan['number'], head=plan['head'],
                 base=plan['base'], evidenceSha256=digest(record),
                 state='write-starting; do not replay', startedAt=now())
    write(receipt, value)
    result = api(f"repos/{plan['repository']}/pulls/{plan['number']}/merge", 'PUT', {
        'sha': plan['head'], 'merge_method': 'squash',
        'commit_message': 'Operator-reviewed local CI equivalence passed. Evidence SHA256: '
                          + digest(record) + '. Hosted CI restrictions waived by operator.'})
    value['response'] = result
    write(receipt, value)
    if result['state'] != 'observed' or not result['data'].get('merged'):
        raise ValueError('Normal GitHub merge not acknowledged; protections remain enforced')
    after = api(f"repos/{plan['repository']}/pulls/{plan['number']}")
    if after['state'] != 'observed' or not after['data'].get('merged'):
        raise ValueError('Merge readback unavailable; do not replay')
    commit = api(f"repos/{plan['repository']}/git/commits/{after['data']['merge_commit_sha']}")
    if commit['state'] != 'observed' or commit['data']['tree']['sha'] != plan['tree']:
        raise ValueError('Merged integration tree changed; retain uncertain receipt for review')
    value.update(state='verified', sha=after['data']['merge_commit_sha'], completedAt=now())
    write(receipt, value)
    print(plan['repository'], plan['number'], 'merged', value['sha'], flush=True)

