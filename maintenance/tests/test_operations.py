import argparse
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from zixcel_github_maintenance.__main__ import apply, digest, write


class MutationBoundary(unittest.TestCase):
    def plan(self, root, operation):
        path = Path(root) / 'plan.json'
        write(path, {'schema': 'zix-github-maintenance-plan/v1', 'operations': [operation]})
        return argparse.Namespace(plan=path, output=Path(root) / 'receipts')

    def test_metadata_drift_does_not_write(self):
        with tempfile.TemporaryDirectory() as root:
            args = self.plan(root, {'repository': 'owner/repo', 'repositoryId': 1, 'kind': 'metadata',
                'expected': {'description': 'old'}, 'desired': {'description': 'new'}})
            with patch('zixcel_github_maintenance.__main__.api', return_value={'state': 'observed',
                'data': {'id': 1, 'permissions': {'admin': True}, 'archived': False, 'description': 'changed'}}) as api:
                with self.assertRaisesRegex(ValueError, 'drift'):
                    apply(args)
                self.assertEqual(api.call_count, 1)

    def test_unknown_write_targets_are_refused(self):
        with tempfile.TemporaryDirectory() as root:
            args = self.plan(root, {'repository': 'owner/repo', 'repositoryId': 1, 'kind': 'metadata',
                'expected': {'private': True}, 'desired': {'private': False}})
            with patch('zixcel_github_maintenance.__main__.api', return_value={'state': 'observed',
                'data': {'id': 1, 'permissions': {'admin': True}, 'archived': False}}) as api:
                with self.assertRaisesRegex(ValueError, 'Unsupported metadata'):
                    apply(args)
                self.assertEqual(api.call_count, 1)

    def test_uncertain_write_is_not_replayed(self):
        with tempfile.TemporaryDirectory() as root:
            args = self.plan(root, {'repository': 'owner/repo', 'repositoryId': 1, 'kind': 'metadata',
                'expected': {'description': 'old'}, 'desired': {'description': 'new'}})
            with patch('zixcel_github_maintenance.__main__.api', side_effect=[{'state': 'observed',
                'data': {'id': 1, 'permissions': {'admin': True}, 'archived': False, 'description': 'old'}},
                {'state': 'unavailable', 'message': 'connection lost'}]):
                with self.assertRaisesRegex(ValueError, 'not acknowledged'):
                    apply(args)
            receipt = json.loads(next(args.output.glob('*.json')).read_text())
            self.assertEqual(receipt['state'], 'write-starting; do not replay')
            with patch('zixcel_github_maintenance.__main__.api') as api:
                with self.assertRaisesRegex(ValueError, 'Prior operation'):
                    apply(args)
                api.assert_not_called()

    def test_ruleset_identity_drift_stops_before_post(self):
        with tempfile.TemporaryDirectory() as root:
            args = self.plan(root, {'repository': 'owner/repo', 'repositoryId': 1, 'kind': 'rulesets-add',
                'expectedRulesetsSha256': digest([]), 'desired': []})
            with patch('zixcel_github_maintenance.__main__.api', return_value={'state': 'observed',
                'data': {'id': 1, 'permissions': {'admin': True}, 'archived': False, 'private': False, 'default_branch': 'main'}}) as api, \
                patch('zixcel_github_maintenance.__main__.pages', return_value={'state': 'observed', 'data': [{'id': 9}]}):
                with self.assertRaisesRegex(ValueError, 'drift'):
                    apply(args)
                self.assertEqual(api.call_count, 1)
