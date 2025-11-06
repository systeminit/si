from typing import NewType, Optional, cast, overload
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

def timestamp_to_sql_timestamp(ts: datetime) -> SqlTimestamp:
    return cast(SqlTimestamp, ts.strftime("%Y-%m-%d %H:%M:%S"))

def timestamp_to_iso_timestamp(ts: datetime) -> IsoTimestamp:
    return cast(IsoTimestamp, ts.strftime("%Y-%m-%dT%H:%M:%SZ"))

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
    if '.' in iso_str:
        iso_timestamp = datetime.strptime(iso_str, "%Y-%m-%dT%H:%M:%S.%fZ")
    else:
        iso_timestamp = datetime.strptime(iso_str, "%Y-%m-%dT%H:%M:%SZ")
    return iso_timestamp.strftime("%Y-%m-%d %H:%M:%S")

@overload
def iso_to_sql_days(iso_str: IsoTimestamp) -> str: ...
@overload
def iso_to_sql_days(iso_str: None) -> None: ...
@overload
def iso_to_sql_days(iso_str: Optional[IsoTimestamp]) -> Optional[str]: ...
def iso_to_sql_days(iso_str: Optional[IsoTimestamp]):
    if iso_str is None:
        return None
    if '.' in iso_str:
        iso_timestamp = datetime.strptime(iso_str, "%Y-%m-%dT%H:%M:%S.%fZ")
    else:
        iso_timestamp = datetime.strptime(iso_str, "%Y-%m-%dT%H:%M:%SZ")
    return iso_timestamp.strftime('%Y-%m-%d')
