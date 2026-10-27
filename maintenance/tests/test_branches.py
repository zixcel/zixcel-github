import copy
import unittest
from zixcel_github_maintenance.branches import merged_pull_proof, protected_names


class BranchProof(unittest.TestCase):
    def setUp(self):
        self.pull = {'number': 1, 'merged_at': '2026-10-28T00:00:00Z',
                     'base': {'ref': 'main'},
                     'head': {'ref': 'feature', 'sha': 'a' * 40, 'repo': {'full_name': 'owner/repo'}}}

    def test_exact_merged_head_only(self):
        self.assertEqual(merged_pull_proof([self.pull], 'owner/repo', 'feature', 'a' * 40, 'main')['number'], 1)
        self.assertIsNone(merged_pull_proof([self.pull], 'owner/repo', 'feature', 'b' * 40, 'main'))

    def test_closed_unmerged_foreign_and_other_base_do_not_authorize_deletion(self):
        for edit in ['unmerged', 'foreign', 'base', 'branch']:
            pull = copy.deepcopy(self.pull)
            if edit == 'unmerged':
                pull['merged_at'] = None
            elif edit == 'foreign':
                pull['head']['repo']['full_name'] = 'foreign/repo'
            elif edit == 'base':
                pull['base']['ref'] = 'other'
            else:
                pull['head']['ref'] = 'other'
            self.assertIsNone(merged_pull_proof([pull], 'owner/repo', 'feature', 'a' * 40, 'main'))

    def test_current_and_conventional_primary_branches_are_never_candidates(self):
        self.assertTrue({'main', 'master', 'develop', 'development', 'custom-default'} <= protected_names('custom-default'))
