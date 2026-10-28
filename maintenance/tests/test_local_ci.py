import os
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from zixcel_github_maintenance.local_ci import execute, observe, merge, prepare
from zixcel_github_maintenance.__main__ import digest, command


class LocalCiTests(unittest.TestCase):
    def test_head_and_base_must_both_match(self):
        plan = dict(repository='owner/repo', number=1, head='a', base='b')
        for head, base, state in [('x', 'b', 'open'), ('a', 'b', 'closed')]:
            with patch('zixcel_github_maintenance.local_ci.api', return_value={
                'state': 'observed', 'data': dict(state=state, head=dict(sha=head), base=dict(sha=base, ref='main'))
            }):
                with self.assertRaises(ValueError):
                    observe(plan)

    def test_current_main_reference_overrides_stale_pr_comparison_base(self):
        plan = dict(repository='owner/repo', number=1, head='a', base='current')
        pr = dict(state='open', head=dict(sha='a'), base=dict(sha='old', ref='main'))
        for current in ['current', 'moved']:
            with patch('zixcel_github_maintenance.local_ci.api', side_effect=[
                dict(state='observed', data=pr),
                dict(state='observed', data=dict(object=dict(sha=current)))
            ]):
                if current == 'current':
                    self.assertEqual(observe(plan), pr)
                else:
                    with self.assertRaises(ValueError):
                        observe(plan)

    def test_failure_is_retained_and_shell_text_is_literal(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / 'check.log'
            code = execute([sys.executable, '-c', 'import sys; print(sys.argv[1]); sys.exit(7)',
                            '$(touch unsafe)'], directory, os.environ.copy(), log, 5)
            self.assertEqual(code, 7)
            self.assertIn('$(touch unsafe)', log.read_text())
            self.assertFalse((Path(directory) / 'unsafe').exists())

    def test_timeout_terminates_child_group(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(subprocess.TimeoutExpired):
                execute([sys.executable, '-c', 'import time; time.sleep(30)'],
                        directory, os.environ.copy(), Path(directory) / 'check.log', 1)

    def test_failed_or_tampered_evidence_cannot_merge(self):
        with tempfile.TemporaryDirectory() as directory:
            evidence = Path(directory) / 'receipt.json'
            args = SimpleNamespace(evidence=evidence, output=Path(directory) / 'merge')
            for state in ['failed', 'passed']:
                plan = {'checks': [{}]}
                evidence.write_text(json.dumps(dict(schema='zix-local-ci-evidence/v1',
                    state=state, plan=plan, planSha256=digest(plan), checks=[dict(exitCode=0, logSha256='wrong')])) )
                (Path(directory) / '000.log').write_text('changed log')
                with patch('zixcel_github_maintenance.local_ci.api') as api:
                    with self.assertRaises(ValueError):
                        merge(args)
                    api.assert_not_called()
            self.assertFalse(args.output.exists())

    def test_prepare_uses_live_main_as_first_parent_and_fresh_runtime(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / 'repo'
            repo.mkdir()
            def git(*args):
                return command(['git', '-c', 'user.name=Test', '-c',
                                'user.email=test@localhost', *args], cwd=repo).stdout.strip()
            git('init', '--initial-branch=main')
            git('remote', 'add', 'origin', 'https://github.com/owner/repo.git')
            (repo / 'base').write_text('original')
            git('add', '.')
            git('commit', '-m', 'base')
            old = git('rev-parse', 'HEAD')
            git('switch', '-c', 'source')
            (repo / 'source').write_text('PR')
            git('add', '.')
            git('commit', '-m', 'PR')
            head = git('rev-parse', 'HEAD')
            git('switch', 'main')
            (repo / 'base').write_text('updated')
            git('commit', '-am', 'new main')
            base = git('rev-parse', 'HEAD')
            plan = dict(schema='zix-local-ci-plan/v1', repository='owner/repo',
                        number=1, checkout=str(repo), head=head, base=old,
                        checks=[dict(name='smoke', argv=['true'], timeoutSeconds=1)],
                        env=dict(RUNNER_TEMP=str(root / 'old'), GITHUB_ENV=str(root / 'old/env')))
            path = root / 'plan.json'
            path.write_text(json.dumps(plan))
            output = root / 'new.json'
            def api(path):
                if '/pulls/' in path:
                    return dict(state='observed', data=dict(state='open', head=dict(sha=head), base=dict(sha=old, ref='main')))
                return dict(state='observed', data=dict(object=dict(sha=base)))
            def local_command(argv, **kwargs):
                if argv[:2] == ['git', 'fetch']:
                    return subprocess.CompletedProcess(argv, 0, '', '')
                return command(argv, **kwargs)
            with patch('zixcel_github_maintenance.local_ci.api', side_effect=api), \
                    patch('zixcel_github_maintenance.local_ci.command', side_effect=local_command):
                prepare(SimpleNamespace(plan=path, output=output))
            result = json.loads(output.read_text())
            self.assertEqual(git('rev-parse', 'HEAD^1'), base)
            self.assertEqual(git('rev-parse', 'HEAD^2'), head)
            self.assertEqual((repo / 'base').read_text(), 'updated')
            self.assertEqual((repo / 'source').read_text(), 'PR')
            self.assertNotEqual(result['env']['RUNNER_TEMP'], plan['env']['RUNNER_TEMP'])
            self.assertTrue(Path(result['env']['GITHUB_ENV']).exists())


if __name__ == '__main__':
    unittest.main()
