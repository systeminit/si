from typing import Iterable, NotRequired, cast
from datetime import date, datetime, timedelta, timezone
import itertools
import logging

from si_posthog_api import PosthogApi
from si_lago_api import ExternalSubscriptionId
from si_lambda import SiLambda, SiLambdaEnv
from si_types import OwnerPk, SqlTimestamp, WorkspaceId, sql_to_iso_timestamp

POSTHOG_EVENT_NAME = "test-billing-workspace_resource_hours"

class UploadPosthogBillingDataEnv(SiLambdaEnv):
    batch_hours: NotRequired[int]

class UploadPosthogBillingData(SiLambda):
    def __init__(self, event: UploadPosthogBillingDataEnv):
        super().__init__(event)
        self.batch_hours = int(event.get("batch_hours", 24*10))
        assert self.batch_hours > 0

    def run(self):
        while self.upload_batch():
            pass

    def upload_batch(self):
        #
        # Upload events
        #
        upload_start, upload_end = self.get_upload_range()
        if upload_start >= upload_end:
            logging.info(f"No more events to upload! upload_range {upload_start} to {upload_end} is empty.")
            return False

        print("Uploading events from {upload_start} to {upload_end}")
        all_events = [
            cast(tuple[WorkspaceId, SqlTimestamp, int, OwnerPk, SqlTimestamp, str], row)
            for row in self.redshift.query_raw(
                f"""
                    SELECT workspace_id, hour_start, resource_count, owner_pk, event_timestamp, owner_si_hours_with_subscriptions.plan_code
                      FROM workspace_operations.workspace_resource_hours
                      LEFT OUTER JOIN workspace_operations.owner_si_hours_with_subscriptions USING (owner_pk, hour_start)
                     CROSS JOIN workspace_operations.workspace_update_events_summary
                     WHERE resource_count > 0
                       AND :upload_start <= hour_start
                       AND hour_start < :upload_end
                     ORDER BY hour_start DESC
                """,
                upload_start=upload_start,
                upload_end=upload_end,
            )
        ]
        self.posthog.post("/batch", {
            "historical_migration": True,
            "batch": [
                {
                    "event": "test-billing-workspace_resource_hours",
                    "properties": {
                        "distinct_id": str(owner_pk),
                        "workspace_id": str(workspace_id),
                        "resource_count": resource_count,
                        "event_timestamp": sql_to_iso_timestamp(event_timestamp),
                        "plan_code": plan_code,
                    },
                    "timestamp": sql_to_iso_timestamp(hour_start),
                }
                for [workspace_id, hour_start, resource_count, owner_pk, event_timestamp, plan_code] in all_events
            ]
        })
        self.update_progress(upload_end)
        logging.info(f"Uploaded {len(all_events)} events to Posthog from {upload_start} to {upload_end}")
        return True

    def get_upload_range(self):
        first_hour_start, last_complete_hour_end = [
            cast(tuple[SqlTimestamp, SqlTimestamp], row)
            for row in self.redshift.query_raw(
                f"""
                    SELECT DATE_TRUNC('hour', first_event) AS first_hour_start, last_complete_hour_end
                    FROM workspace_operations.workspace_update_events_summary
                """)
        ][0]
        # Start the upload where we last left off, or at the beginning of time
        uploaded_to = [
            cast(SqlTimestamp, uploaded_to)
            for [uploaded_to] in self.redshift.query_raw(f"""
                SELECT uploaded_to
                FROM workspace_operations.upload_progress
                WHERE upload_type = 'posthog-workspace_resource_counts'
            """)
        ]
        batch_start = uploaded_to[0] if len(uploaded_to) > 0 else first_hour_start
        # End the batch at the last complete hour, or the max batch size, whichever comes first
        batch_end = [
            cast(SqlTimestamp, batch_end)
            for [batch_end] in self.redshift.query_raw(
                f"""
                    -- I can't believe this is the simplest way to get the maximum of two values
                    SELECT MIN(batch_end) AS batch_end FROM (
                        SELECT DATEADD(HOUR, {self.batch_hours}, :batch_start::timestamp) AS batch_end
                        UNION
                        SELECT :last_complete_hour_end::timestamp AS batch_end
                    )
                """,
                batch_start=batch_start,
                last_complete_hour_end=last_complete_hour_end
            )
        ][0]
        return (batch_start, batch_end)

    def update_progress(self, uploaded_to: SqlTimestamp):
       self.redshift.execute(
           f"""
                -- There doesn't seem to be a nicer way to INSERT OR UPDATE in Redshift
                MERGE INTO workspace_operations.upload_progress
                    USING (SELECT
                        'posthog-workspace_resource_counts' AS upload_type,
                        :uploaded_to::timestamp AS uploaded_to
                    ) AS my_source
                    ON upload_progress.upload_type = my_source.upload_type
                    WHEN MATCHED THEN UPDATE SET uploaded_to = my_source.uploaded_to
                    WHEN NOT MATCHED THEN INSERT (upload_type, uploaded_to) VALUES (my_source.upload_type, my_source.uploaded_to)
            """,
            uploaded_to=uploaded_to
        )

lambda_handler = UploadPosthogBillingData.lambda_handler

lambda_handler()