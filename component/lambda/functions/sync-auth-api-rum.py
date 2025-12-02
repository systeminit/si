#!/usr/bin/env python3
"""
Lambda to sync workspace RUM (Resources Under Management) data from Redshift to Auth API.

This lambda queries Redshift for the maximum resource count for each workspace
for the current month and last month, then uploads that data to the Auth API
via the bulk upsert endpoint.

The lambda is idempotent - it can be run multiple times without creating duplicate
data, and will repair any missing or corrupted data from previous runs.
"""

from typing import Iterable, TypedDict, cast
import logging

from si_lambda import SiLambda, SiLambdaEnv
from si_types import Optional, OwnerPk, WorkspaceId, SqlTimestamp

UPLOAD_PROGRESS_TYPE = 'auth-rum-sync'
EXPUNGED_WORKSPACES = ["01J9J0RNFZTBV80B20ZT11QM66"]

class SyncWorkspaceRumEnv(SiLambdaEnv):
    """Environment variables specific to the RUM sync lambda"""
    pass

class SyncWorkspaceRum(SiLambda):
    def __init__(self, event: SyncWorkspaceRumEnv):
        super().__init__(event)
        self.batch_hours = int(event.get("batch_hours", 30*24))
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

        #
        # Get the RUM change events to upload
        #
        rum_changes = cast(
            Iterable[RumChange],
            self.redshift.query(
                f"""
                    SELECT *
                      FROM workspace_operations.rum_changes
                     WHERE (:batch_start <= event_timestamp AND event_timestamp < :batch_end)
                        -- Also check if the end event falls in the range, to update the end event timestamp
                        OR (:batch_start <= next_workspace_event_timestamp AND next_workspace_event_timestamp < :batch_end)
                     ORDER BY owner_pk, event_timestamp ASC
                """,
                batch_start=batch_start,
                batch_end=batch_end,
            )
        )

        rum_change_events = [
            {
                "eventTimestamp": rum_change["event_timestamp"],
                "rumChange": rum_change["rum_change"],
                "workspaceId": str(rum_change["workspace_id"]),
                "workspaceRum": rum_change["workspace_rum"],
                "nextWorkspaceEventTimestamp": rum_change["next_workspace_event_timestamp"],
                "prevWorkspaceEventTimestamp": rum_change["prev_workspace_event_timestamp"],
                "ownerId": str(rum_change["owner_pk"]),
                "ownerRum": rum_change["owner_rum"],
                "nextOwnerEventTimestamp": rum_change["next_owner_event_timestamp"],
                "prevOwnerEventTimestamp": rum_change["prev_owner_event_timestamp"],
            }
            for rum_change in rum_changes
            if rum_change["owner_pk"] is not None and rum_change["workspace_id"] not in EXPUNGED_WORKSPACES
        ]

        if self.dry_run:
            logging.info(f"[DRY RUN] Would upload {len(rum_change_events)} RUM records:")
            for record in rum_change_events[:5]:  # Show first 5 records
                logging.info(f"  {record}")
            if len(rum_change_events) > 5:
                logging.info(f"  ... and {len(rum_change_events) - 5} more")
            return False

        if len(rum_change_events) == 0:
            logging.info(f"No RUM changes to upload from {batch_start} to {batch_end}.")
        else:
            logging.info(f"Uploading {len(rum_change_events)} RUM records to Auth API")
            response = self.auth_api.post("/rum-data/bulk-upsert", json={"data": rum_change_events})
            result = response.json()
            logging.info(f"Successfully uploaded {result.get('recordsProcessed', 0)} records from {batch_start} to {batch_end}")


        self.update_upload_progress(UPLOAD_PROGRESS_TYPE, batch_end)

        return True

lambda_handler = SyncWorkspaceRum.lambda_handler

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
