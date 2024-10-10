import json
from typing import Optional, cast
from si_redshift import FieldValue, Redshift
from si_types import WorkspaceId, SqlTimestamp

redshift = Redshift.from_env()


class WorkspaceResourceCountEvent:
    @classmethod
    def from_row(
        cls,
        row: list[FieldValue],
    ):
        return cls(*cast(tuple[WorkspaceId, int, SqlTimestamp], row))

    def __init__(
        self,
        workspace_id: WorkspaceId,
        resource_count: int,
        event_timestamp: SqlTimestamp,
    ):
        self.workspace_id = workspace_id
        self.resource_count = resource_count
        self.event_timestamp = event_timestamp


class WorkspaceResourceCounts:
    def __init__(self, redshift: Redshift, at_time: Optional[SqlTimestamp] = None):
        self.redshift = redshift
        self.last_event: Optional[SqlTimestamp] = None
        self.resource_count = dict[WorkspaceId, int]()
        for count in self._get_workspace_resource_counts(at_time):
            self.set(count)

    def get(self, workspace_id: WorkspaceId) -> Optional[int]:
        return self.resource_count.get(workspace_id)

    def for_workspace(self, workspace_id: WorkspaceId) -> Optional[int]:
        return self.resource_count.get(workspace_id)

    def set(self, event: WorkspaceResourceCountEvent):
        self.resource_count[event.workspace_id] = event.resource_count
        self.last_event = event.event_timestamp

    def _get_workspace_resource_counts(self, at_time: Optional[SqlTimestamp]):
        params: dict[str, FieldValue]
        if at_time is None:
            where_clause = "WHERE next_event_timestamp IS NULL"
            params = {}
        else:
            where_clause = """
                WHERE next_event_timestamp <= :at_time
                  AND (:at_time < next_event_timestamp OR next_event_timestamp IS NULL)
            """
            params = {"at_time": at_time}

        return map(
            WorkspaceResourceCountEvent.from_row,
            redshift.query_raw(
                f"""
            SELECT workspace_id, resource_count, event_timestamp
              FROM workspace_operations.workspace_resource_counts
             {where_clause}
             ORDER BY event_timestamp
        """,
                **params,
            ),
        )


# def workspace_resource_count_history(redshift: Redshift, start_time: SqlTimestamp):
#     return cast(Iterator[tuple[WorkspaceId, int, str]], redshift.execute())


def insert_workspace_resource_count(
    redshift: Redshift, event: WorkspaceResourceCountEvent
):
    # Insert the record (and update the old record to point at the new record's range)
    redshift.execute(
        """
        BEGIN;

        UPDATE workspace_operations.workspace_resource_counts
            SET next_event_timestamp = :event_timestamp
            WHERE workspace_id = :workspace_id AND next_event_timestamp IS NULL;

        INSERT INTO workspace_operations.workspace_resource_counts
            (workspace_id, resource_count, event_timestamp, next_event_timestamp)
        VALUES
            (:workspace_id, :resource_count, :event_timestamp, NULL);

        COMMIT;
    """,
        workspace_id=event.workspace_id,
        resource_count=str(event.resource_count),
        event_timestamp=event.event_timestamp,
    )


def complete_workspace_update_events(
    redshift: Redshift, start_time: Optional[SqlTimestamp]
):
    """
    Get workspace update events that are definitely finished (i.e. we do not expect any more
    events to show up later within the same time window).
    """
    end_time = cast(
        SqlTimestamp,
        next(
            redshift.query_raw(
                """
        SELECT DATEADD(MINUTE, -15, CAST(MAX(event_timestamp) AS TIMESTAMP))
                  FROM workspace_update_events.workspace_update_events
    """
            )
        )[0],
    )

    if start_time:
        print(f"Getting workspace_update_events from {start_time} to {end_time} ...")
    else:
        print(f"Getting ALL workspace_update_events up to {end_time} ...")

    return map(
        WorkspaceResourceCountEvent.from_row,
        redshift.query_raw(
            f"""
            SELECT workspace_id, resource_count, event_timestamp
            FROM workspace_update_events.workspace_update_events
            WHERE CAST(event_timestamp AS TIMESTAMP) < :end_time
              {"AND CAST(event_timestamp AS TIMESTAMP) >= :start_time" if start_time else ""}
              AND resource_count IS NOT NULL
            ORDER BY event_timestamp
        """,
            end_time=end_time,
            **({"start_time": start_time} if start_time else {}),
        ),
    )


def lambda_handler(_event=None, _context=None):
    # UPDATE WORKSPACE_RESOURCE_COUNTS
    # 1. Determine start_time based on latest workspace_resource_count.
    # 2. Determine end_time based on latest workspace_update_event - 15 minutes.
    # 3. Initialize workspace_resource_count from workspace_resource_counts where next_event_timestamp = NULL (last event).
    # 4. UPDATE WORKSPACE_RESOURCE_COUNTS: for each workspace_update_event from start_time to end_time:
    #    a. If resource_count is null or the same, skip it.
    #    b. Update workspace_resource_count from NULL to next_event_timestamp.
    #    c. Insert new workspace_resource_count with event_timestamp.

    resource_counts = WorkspaceResourceCounts(redshift)

    # Update workspace_resource_counts by adding a record each time it changes
    for event in complete_workspace_update_events(redshift, resource_counts.last_event):
        if event.resource_count != resource_counts.for_workspace(event.workspace_id):
            print(f"Event: {event.event_timestamp}")
            resource_counts.set(event)
            insert_workspace_resource_count(redshift, event)

    return {"statusCode": 200, "body": json.dumps({})}


lambda_handler(None, None)
