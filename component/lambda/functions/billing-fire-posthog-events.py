from typing import Iterable, NotRequired, Optional, TypedDict, cast
import logging

from si_lambda import SiLambda, SiLambdaEnv
from si_posthog_api import PosthogApi
from si_types import OwnerPk, SqlTimestamp, WorkspaceId, sql_to_iso_timestamp

class UploadPosthogBillingDataEnv(SiLambdaEnv):
    batch_hours: NotRequired[int]

UPLOAD_PROGRESS_TYPE = 'posthog-billing-rum-changed'
POSTHOG_EVENT_TYPE = 'billing-rum-changed'

class UploadPosthogBillingData(SiLambda):
    def __init__(self, event: UploadPosthogBillingDataEnv):
        super().__init__(event)
        self.batch_hours = int(event.get("batch_hours", 24*30))
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
            Iterable[RumChange],
            self.redshift.query(
                f"""
                    SELECT *
                      FROM workspace_operations.rum_changes
                     WHERE :batch_start <= event_timestamp
                       AND event_timestamp < :batch_end
                     ORDER BY owner_pk, event_timestamp ASC
                """,
                batch_start=batch_start,
                batch_end=batch_end,
            )
        )

        rum_change_events: list[PosthogApi.BatchEvent] = [
            {
                "event": POSTHOG_EVENT_TYPE,
                "properties": {
                    "distinct_id": str(rum_change["owner_pk"]),
                    "rum_change": rum_change["rum_change"],
                    "owner_rum": rum_change["owner_rum"],
                    "$set": { "rum": rum_change["owner_rum"] },
                    # We don't report the next change time, because we can't update the event after the fact and we might
                    # not *know* the next change time yet. But the previous change time is still useful.
                    "prev_owner_rum_change": sql_to_iso_timestamp(rum_change["prev_owner_event_timestamp"]),
                    "workspace_id": str(rum_change["workspace_id"]),
                    "workspace_rum": rum_change["workspace_rum"],
                    "prev_workspace_rum_change": sql_to_iso_timestamp(rum_change["prev_workspace_event_timestamp"]),
                },
                "timestamp": sql_to_iso_timestamp(rum_change["event_timestamp"]),
            }
            for rum_change in rum_changes
            if rum_change["owner_pk"] is not None
        ]
        logging.info(f"Got {len(rum_change_events)} {POSTHOG_EVENT_TYPE} events.")

        #
        # Upload the events to Posthog
        #
        all_events = rum_change_events
        logging.info(f"Got {len(all_events)} events. Uploading to Posthog ...")
        historical_migration = batch_end != last_complete_hour_end
        if len(all_events) > 0:
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
            for [uploaded_to] in self.redshift.query_raw(
                f"""
                    SELECT uploaded_to
                    FROM workspace_operations.upload_progress
                    WHERE upload_type = :upload_type
                """,
                upload_type=UPLOAD_PROGRESS_TYPE
            )
        ]
        batch_start = uploaded_to[0] if len(uploaded_to) > 0 else first_hour_start
        # End the batch at the last complete hour, or the max batch size, whichever comes first
        batch_end = [
            cast(SqlTimestamp, batch_end)
            for [batch_end] in self.redshift.query_raw(
                f"""
                    SELECT LEAST(
                               DATEADD(HOUR, {self.batch_hours}, :batch_start::timestamp),
                               :last_complete_hour_end::timestamp
                           ) AS batch_end
                """,
                batch_start=batch_start,
                last_complete_hour_end=last_complete_hour_end
            )
        ][0]
        return (batch_start, batch_end)

    def update_progress(self, uploaded_to: SqlTimestamp):
        if self.dry_run:
            logging.info(f"Dry run: not updating upload progress to {uploaded_to}")
            return
        self.redshift.execute(
            f"""
                -- There doesn't seem to be a nicer way to INSERT OR UPDATE in Redshift
                MERGE INTO workspace_operations.upload_progress
                    USING (SELECT
                        :upload_type::text AS upload_type,
                        :uploaded_to::timestamp AS uploaded_to
                    ) AS my_source
                    ON upload_progress.upload_type = my_source.upload_type
                    WHEN MATCHED THEN UPDATE SET uploaded_to = my_source.uploaded_to
                    WHEN NOT MATCHED THEN INSERT (upload_type, uploaded_to) VALUES (my_source.upload_type, my_source.uploaded_to)
            """,
            upload_type=UPLOAD_PROGRESS_TYPE,
            uploaded_to=uploaded_to
        )

lambda_handler = UploadPosthogBillingData.lambda_handler

class RumChange(TypedDict):
    event_timestamp: SqlTimestamp
    rum_change: int
    workspace_id: WorkspaceId
    workspace_rum: int
    prev_workspace_event_timestamp: Optional[SqlTimestamp]
    next_workspace_event_timestamp: Optional[SqlTimestamp]
    owner_pk: Optional[OwnerPk]
    owner_rum: int
    prev_owner_event_timestamp: Optional[SqlTimestamp]
    next_owner_event_timestamp: Optional[SqlTimestamp]
