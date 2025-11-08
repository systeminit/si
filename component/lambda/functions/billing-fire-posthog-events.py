from typing import Iterable, NotRequired, Optional, cast
import logging

from si_lambda import SiLambda, SiLambdaEnv
from si_posthog_api import PosthogApi
from si_types import OwnerPk, SqlTimestamp, WorkspaceId, sql_to_iso_timestamp

class UploadPosthogBillingDataEnv(SiLambdaEnv):
    batch_hours: NotRequired[int]

class UploadPosthogBillingData(SiLambda):
    def __init__(self, event: UploadPosthogBillingDataEnv):
        super().__init__(event)
        self.batch_hours = int(event.get("batch_hours", 24*10))
        assert self.batch_hours > 0

    def run(self):
        (first_hour_start, last_complete_hour_end) = self.get_full_event_range()
        while self.upload_batch(first_hour_start, last_complete_hour_end):
            pass

    def upload_batch(self, first_hour_start: SqlTimestamp, last_complete_hour_end: SqlTimestamp):
        #
        # Figure out the time range of events to upload
        #
        batch_start, batch_end = self.get_upload_range(first_hour_start, last_complete_hour_end)
        if batch_start >= batch_end:
            logging.info(f"No more events to upload! upload_range {batch_start} to {batch_end} is empty.")
            return False
        logging.info(f"Fetching events from {batch_start} to {batch_end}")

        #
        # Get the events to upload
        #

        # We run the queries in parallel because they individually take 2+ minutes each, and
        # their results don't depend on each other
        rum_changes = cast(
            Iterable[tuple[OwnerPk, SqlTimestamp, int, Optional[int]]],
            self.redshift.query_raw(
                f"""
                    SELECT owner_pk, sample_time, resource_count, prev_resource_count
                      FROM workspace_operations.owner_resource_hours
                     WHERE (resource_count <> prev_resource_count OR prev_resource_count IS NULL)
                       AND :batch_start <= hour_start
                       AND hour_start < :batch_end
                """,
                batch_start=batch_start,
                batch_end=batch_end,
            )
        )
        owner_billing_months = cast(
            Iterable[tuple[OwnerPk, SqlTimestamp, int, int, int]],
            self.redshift.query_raw(
                f"""
                    SELECT owner_pk, month, max_resource_count, resource_hours, is_free
                      FROM workspace_operations.owner_billing_months
                     WHERE :batch_start <= month AND month < :batch_end
                """,
                batch_start=batch_start,
                batch_end=batch_end,
            )
        )
        # TODO remove and stop uploading these when reports have migrated off them
        workspace_resource_hours = cast(
            Iterable[tuple[WorkspaceId, SqlTimestamp, int, OwnerPk, SqlTimestamp, str]],
            self.redshift.query_raw(
                f"""
                    SELECT workspace_id, hour_start, resource_count, owner_pk, event_timestamp, owner_si_hours_with_subscriptions.plan_code
                      FROM workspace_operations.workspace_resource_hours
                      LEFT OUTER JOIN workspace_operations.owner_si_hours_with_subscriptions USING (owner_pk, hour_start)
                     WHERE :batch_start <= hour_start AND hour_start < :batch_end
                       AND resource_count > 0
                """,
                batch_start=batch_start,
                batch_end=batch_end,
            )
        )

        rum_change_events: list[PosthogApi.BatchEvent] = [
            {
                "event": "billing-rum_changed",
                "properties": {
                    "distinct_id": str(owner_pk),
                    "resource_count": resource_count,
                    "prev_resource_count": prev_resource_count
                },
                "timestamp": sql_to_iso_timestamp(sample_time),
            }
            for [owner_pk, sample_time, resource_count, prev_resource_count] in rum_changes
        ]
        logging.info(f"Got {len(rum_change_events)} billing-rum_changed events.")
        owner_billing_month_events: list[PosthogApi.BatchEvent] = [
            {
                "event": "billing-owner_billing_month",
                "properties": {
                    "distinct_id": str(owner_pk),
                    "max_resource_count": max_resource_count,
                    "resource_hours": resource_hours,
                    "is_free": bool(is_free),
                },
                "timestamp": sql_to_iso_timestamp(month),
            }
            for [owner_pk, month, max_resource_count, resource_hours, is_free] in owner_billing_months
        ]
        logging.info(f"Got {len(owner_billing_month_events)} billing-owner_billing_month events.")
        workspace_resource_hours_events: list[PosthogApi.BatchEvent] = [
            {
                "event": "billing-workspace_resource_hours",
                "properties": {
                    "distinct_id": str(owner_pk),
                    "workspace_id": str(workspace_id),
                    "resource_count": resource_count,
                    "event_timestamp": sql_to_iso_timestamp(event_timestamp),
                    "plan_code": plan_code,
                },
                "timestamp": sql_to_iso_timestamp(hour_start),
            }
            for [workspace_id, hour_start, resource_count, owner_pk, event_timestamp, plan_code] in workspace_resource_hours
        ]
        logging.info(f"Got {len(workspace_resource_hours_events)} billing-workspace_resource_hours events.")

        #
        # Upload the events to Posthog
        #
        all_events = rum_change_events + owner_billing_month_events + workspace_resource_hours_events
        logging.info(f"Got {len(all_events)} events. Uploading to Posthog ...")
        historical_migration = batch_end != last_complete_hour_end
        self.posthog.post("/batch", {
            "historical_migration": historical_migration,
            "batch": all_events
        })

        self.update_progress(batch_end)

        logging.info(f"Uploaded {len(all_events)} events to Posthog from {batch_start} to {batch_end} with historical_migration={historical_migration}")

        return True

    def get_full_event_range(self):
        first_hour_start, last_complete_hour_end = [
            cast(tuple[SqlTimestamp, SqlTimestamp], row)
            for row in self.redshift.query_raw(
                f"""
                    SELECT DATE_TRUNC('hour', first_event) AS first_hour_start, last_complete_hour_end
                    FROM workspace_operations.workspace_update_events_summary
                """)
        ][0]
        return (first_hour_start, last_complete_hour_end)

    def get_upload_range(self, first_hour_start: SqlTimestamp, last_complete_hour_end: SqlTimestamp):
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
