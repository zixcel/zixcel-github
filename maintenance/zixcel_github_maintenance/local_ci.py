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

from .__main__ import api, command, digest, now, owner_repository, write


def observe(plan):
    owner_repository(plan['repository'])
    result = api(f"repos/{plan['repository']}/pulls/{plan['number']}")
    if result['state'] != 'observed':
        raise ValueError('PR observation unavailable')
    pr = result['data']
    if (pr['state'] != 'open' or pr['head']['sha'] != plan['head']
            or pr['base']['sha'] != plan['base']):
        raise ValueError('PR head/base drift or closed PR')
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

