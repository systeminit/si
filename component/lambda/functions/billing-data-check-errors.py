import logging, typing
from typing import cast, Iterable, NotRequired, Literal, LiteralString, TypeVar

from si_lambda import SiLambda, SiLambdaEnv

IssueQuery = Literal[
    'subscription_issues',
    'subscription_count_issues',
    'workspace_issues',
    'workspace_update_events_before_subscriptions',
    'owner_resource_hours_without_subscriptions'
]
ALL_ISSUE_QUERIES: list[IssueQuery] = list(typing.get_args(IssueQuery))

Issue = Literal[
    'null_plan_code',
    'no_start_time',
    'start_after_end',
    'unbounded_free_trial',
    'free_trial_not_first',
    'gap_in_subscriptions',
    'overlapping_subscriptions',
    'multiple_unbounded_subscriptions',
    'no_subscriptions',
    'no_free_trial',
    'no_unbounded_subscription',
    'multiple_unbounded_subscriptions',
    'owner_change',
    'no_owner',
    'event_before_subscription',
    'events_without_subscription',
]
ALL_ISSUES: list[Issue] = list(typing.get_args(Issue))


class BillingDataCheckErrorsEnv(SiLambdaEnv):
    issue_queries: NotRequired[list[IssueQuery]]
    ignore_issues: NotRequired[list[Issue]]

class BillingDataCheckErrors(SiLambda):
    def __init__(self, event: BillingDataCheckErrorsEnv):
        super().__init__(event)
        self.issue_queries = event.get('issue_queries', [
            query
            for query in ALL_ISSUE_QUERIES
            if query not in [
                'workspace_update_events_before_subscriptions',
                'owner_resource_hours_without_subscriptions'
            ]
        ])
        self.ignore_issues = event.get('ignore_issues', ['no_subscriptions', 'overlapping_subscriptions'])

    def run_issue_query(self, issue_query: IssueQuery):
        ignore_issues = [f"'{issue}'" for issue in self.ignore_issues]
        issue_filter = f" WHERE issue NOT IN ({', '.join(ignore_issues)})" if len(self.ignore_issues) > 0 else ""
        issue_count = 0
        for issue in self.redshift.query(f"SELECT * FROM workspace_verifications.{issue_query}{issue_filter}"):
            logging.error(f"Found issue {issue['issue']} in {issue_query}: {issue}")
            issue_count += 1
        return issue_count

    def run(self):
        issue_count = 0
        for issue_query in self.issue_queries:
            issue_count += self.run_issue_query(issue_query)
        if issue_count > 0:
            logging.error(f"{issue_count} issues found")
        else:
            logging.info("No issues found")

lambda_handler = BillingDataCheckErrors.lambda_handler