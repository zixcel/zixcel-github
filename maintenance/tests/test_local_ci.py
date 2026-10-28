import os
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from zixcel_github_maintenance.local_ci import execute, observe, merge
from zixcel_github_maintenance.__main__ import digest


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


if __name__ == '__main__':
    unittest.main()
