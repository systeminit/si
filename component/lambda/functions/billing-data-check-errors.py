import logging, typing
from os import getenv
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
    ignore_users: NotRequired[list[str]]

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
        self.ignore_users = [s for s in self.getenv('ignore_users', '').split(',') if s != '']

    def run_issue_query(self, issue_query: IssueQuery):
        filters = []

        ignore_issues = [f"'{issue}'" for issue in self.ignore_issues]
        if len(ignore_issues) > 0:
            filters.append(f"issue NOT IN ({', '.join(ignore_issues)})")

        ignore_users = [f"'{user}'" for user in self.ignore_users]
        if len(ignore_users) > 0:
            filters.append(f"owner_pk NOT IN ({', '.join(ignore_users)})")

        issue_filter = f" WHERE {' AND '.join(filters)}" if len(filters) > 0 else ""

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