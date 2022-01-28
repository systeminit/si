export enum ResourceHealth {
  Ok = "ok",
  Warning = "warning",
  Error = "error",
  Unknown = "unknown",
}

export enum ResourceStatus {
  Pending = "pending",
  InProgress = "inProgress",
  Created = "created",
  Failed = "failed",
  Deleted = "deleted",
}

export interface Resource {
  id: number;
  // unixTimestamp: string;
  createdAt: string;
  updatedAt: string;
  error?: string; // TODO: what is this?
  data: unknown; // TODO: what is this?
  // state: any;
  // status: ResourceStatus;
  health: ResourceHealth;
  // systemId: string;
  // nodeId: string;
  // entityId: string;
  entityType: string; // TODO: make it an enum?
  // siStorable: ISiStorable;
}
