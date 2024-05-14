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
  payload: unknown;
  status: ResourceHealth | null;
  message: string | null;
  logs: string[];
  lastSynced?: string;
}

export interface OutputStream {
  line: string;
  stream: string;
  level: string;
}
