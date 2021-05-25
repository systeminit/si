import { SiStorable } from "./siStorable";

export enum ResourceInternalHealth {
  Ok = "ok",
  Warning = "warning",
  Error = "error",
  Unknown = "unknown",
}

export enum ResourceInternalStatus {
  Pending = "pending",
  InProgress = "inProgress",
  Created = "created",
  Failed = "failed",
  Deleted = "deleted",
}

export interface SubResource {
  unixTimestamp: number;
  timestamp: string;
  state: string;
  health: string;
  internalStatus: ResourceInternalStatus;
  internalHealth: ResourceInternalHealth;
  data: Record<string, unknown>;
  error?: string;
}

export interface Resource {
  id: string;
  unixTimestamp: number;
  timestamp: string;
  state: string;
  health: string;
  internalStatus: ResourceInternalStatus;
  internalHealth: ResourceInternalHealth;
  data: Record<string, unknown>;
  subResources: Record<string, SubResource>;
  systemId: string;
  entityId: string;
  error?: string;
  siStorable: SiStorable;
}
