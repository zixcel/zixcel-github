import copy
import unittest
from zixcel_github_maintenance.__main__ import pull_eligibility


class MergeAdmission(unittest.TestCase):
    def setUp(self):
        self.pull = {'isDraft': False, 'mergeable': 'MERGEABLE', 'mergeStateStatus': 'CLEAN',
                     'reviewDecision': '', 'headRefOid': 'a' * 40,
                     'statusCheckRollup': [{'__typename': 'CheckRun', 'name': 'functional',
                         'status': 'COMPLETED', 'conclusion': 'SUCCESS'}]}

    def test_green_exact_head(self):
        self.assertTrue(pull_eligibility(self.pull)['eligible'])

    def test_pending_failed_and_absent_checks_refused(self):
        for checks in [[], [{'status': 'IN_PROGRESS'}], [{'status': 'COMPLETED', 'conclusion': 'FAILURE'}],
                       [{'status': 'COMPLETED', 'conclusion': 'SKIPPED'}]]:
            pull = copy.deepcopy(self.pull)
            pull['statusCheckRollup'] = checks
            self.assertFalse(pull_eligibility(pull)['eligible'])

    def test_draft_conflict_reviews_and_blocked_merge_refused(self):
        for field, value in [('isDraft', True), ('mergeable', 'CONFLICTING'),
                             ('mergeStateStatus', 'BLOCKED'), ('reviewDecision', 'CHANGES_REQUESTED'),
                             ('reviewDecision', 'REVIEW_REQUIRED'), ('headRefOid', 'short')]:
            pull = {**self.pull, field: value}
            self.assertFalse(pull_eligibility(pull)['eligible'])

    def test_legacy_status_failure_is_not_hidden_by_passing_checks(self):
        self.pull['statusCheckRollup'].append({'__typename': 'StatusContext', 'context': 'external', 'state': 'FAILURE'})
        self.assertFalse(pull_eligibility(self.pull)['eligible'])


if __name__ == '__main__':
    unittest.main()
