export type ResourceHealth = "ok" | "warning" | "error" | "unknown";

export type ResourceStatus =
  | "pending"
  | "inProgress"
  | "created"
  | "failed"
  | "deleted";

export interface Resource {
  payload: unknown;
  status: ResourceHealth | null;
  message: string | null;
  lastSynced?: string;
}

export interface OutputStream {
  line: string;
  stream: string;
  level: string;
}
