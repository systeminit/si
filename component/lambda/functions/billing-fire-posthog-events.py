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
        while self.upload_batch():
            pass

    def upload_batch(self):
        #
        # Figure out the time range of events to upload
        #
        batch_start, batch_end, last_complete_hour_end = self.get_upload_range(UPLOAD_PROGRESS_TYPE, self.batch_hours)
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

        self.update_upload_progress(UPLOAD_PROGRESS_TYPE, batch_end)

        logging.info(f"Uploaded {len(all_events)} events to Posthog from {batch_start} to {batch_end} with historical_migration={historical_migration}")

        return True

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
