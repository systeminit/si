import { SiStorable } from "./siStorable";

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
  id: string;
  unixTimestamp: number;
  timestamp: string;
  state: Record<string, unknown>;
  status: ResourceStatus;
  health: ResourceHealth;
  systemId: string;
  entityId: string;
  siStorable: SiStorable;
}
