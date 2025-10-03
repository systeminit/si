from typing import NewType, Optional, overload
from datetime import datetime

Ulid = NewType("Ulid", str)
OwnerPk = NewType("OwnerPk", Ulid)
ChangeSetId = NewType("ChangeSetId", Ulid)
ComponentId = NewType("ComponentId", Ulid)
WorkspaceId = NewType("WorkspaceId", Ulid)
# SQL datetime, i.e. 2024-04-03 12:00:00
SqlDatetime = NewType("SqlDatetime", str)
# SQL timestamp, i.e. 2024-04-03 12:00:00[.000000].
SqlTimestamp = NewType("SqlTimestamp", str)
# ISO 8601 timestamp, i.e. 2024-04-03T12:00:00[.000000]Z
IsoTimestamp = NewType("IsoTimestamp", str)

SQL_TIMESTAMP_FORMAT = "%Y-%m-%d %H:%M:%S.%f"
ISO_TIMESTAMP_FORMAT = "%Y-%m-%dT%H:%M:%SZ"

# Convert ISO 8601 timestamp to the required format
@overload
def sql_to_iso_timestamp(sql_str: SqlTimestamp) -> IsoTimestamp: ...
@overload
def sql_to_iso_timestamp(sql_str: None) -> None: ...
@overload
def sql_to_iso_timestamp(sql_str: Optional[SqlTimestamp]) -> Optional[IsoTimestamp]: ...
def sql_to_iso_timestamp(sql_str: Optional[SqlTimestamp]):
    if sql_str is None:
        return None
    if '.' in sql_str:
        return datetime.strptime(sql_str, "%Y-%m-%d %H:%M:%S.%f").isoformat() + "Z"
    else:
        return datetime.strptime(sql_str, "%Y-%m-%d %H:%M:%S").isoformat() + "Z"

# Convert ISO 8601 timestamp to the required format
@overload
def iso_to_sql_datetime(iso_str: IsoTimestamp) -> SqlDatetime: ...
@overload
def iso_to_sql_datetime(iso_str: None) -> None: ...
@overload
def iso_to_sql_datetime(iso_str: Optional[IsoTimestamp]) -> Optional[SqlDatetime]: ...
def iso_to_sql_datetime(iso_str: Optional[IsoTimestamp]):
    if iso_str is None:
        return None
    return datetime.strptime(iso_str, "%Y-%m-%dT%H:%M:%SZ").strftime("%Y-%m-%d %H:%M:%S")

@overload
def iso_to_sql_days(iso_str: IsoTimestamp) -> str: ...
@overload
def iso_to_sql_days(iso_str: None) -> None: ...
@overload
def iso_to_sql_days(iso_str: Optional[IsoTimestamp]) -> Optional[str]: ...
def iso_to_sql_days(iso_str: Optional[IsoTimestamp]):
    if iso_str is None:
        return None
    return datetime.strptime(iso_str, "%Y-%m-%dT%H:%M:%SZ").strftime('%Y-%m-%d')
