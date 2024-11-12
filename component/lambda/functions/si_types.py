from typing import NewType

Ulid = NewType("Ulid", str)
OwnerPk = NewType("OwnerPk", Ulid)
WorkspaceId = NewType("WorkspaceId", Ulid)
SqlTimestamp = NewType("SqlTimestamp", str)
IsoTimestamp = NewType("IsoTimestamp", str)