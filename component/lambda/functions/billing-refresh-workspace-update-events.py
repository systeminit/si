import logging
from typing import cast
from si_types import SqlTimestamp
from si_lambda import SiLambda, SiLambdaEnv

class BillingRefreshWorkspaceUpdateEnv(SiLambdaEnv):
    pass

class BillingRefreshWorkspaceUpdate(SiLambda):
    def __init__(self, event: BillingRefreshWorkspaceUpdateEnv):
        super().__init__(event)

    def run(self):
        logging.info(f"BEFORE: {self.event_summary()}")
        self.redshift.execute("REFRESH MATERIALIZED VIEW workspace_operations.workspace_update_events_clean")
        logging.info(f"AFTER: {self.event_summary()}")
    
    def event_summary(self):
        event_count, first_event_timestamp, last_event_timestamp, workspace_count = [
            cast(tuple[int, SqlTimestamp, SqlTimestamp, int], row)
            for row in self.redshift.query_raw(
                f"""
                    SELECT COUNT(*) AS event_count,
                           MIN(event_timestamp) AS first_event_timestamp,
                           MAX(event_timestamp) AS last_event_timestamp,
                           COUNT(DISTINCT workspace_id) AS workspace_count
                    FROM workspace_operations.workspace_update_events_clean
                """)
        ][0]
        return {
            "event_count": event_count,
            "first_event_timestamp": first_event_timestamp,
            "last_event_timestamp": last_event_timestamp,
            "workspace_count": workspace_count,
        }

lambda_handler = BillingRefreshWorkspaceUpdate.lambda_handler