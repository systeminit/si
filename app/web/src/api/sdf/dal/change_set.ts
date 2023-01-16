import { StandardModelNoVisibility } from "@/api/sdf/dal/standard_model";

export enum ChangeSetStatus {
  Open = "Open",
  Closed = "Closed",
  Abandoned = "Abandoned",
  Applied = "Applied",
  Failed = "Failed",
}

export interface ChangeSet extends StandardModelNoVisibility {
  id: never;
  name: string;
  note?: string;
  status: ChangeSetStatus;
}

export type ChangeStatus = "added" | "deleted" | "modified" | "unmodified";

export interface ComponentStatsGroup {
  componentId: string;
  componentName: string;
  componentStatus: ChangeStatus;
}

export interface ComponentStats {
  stats: ComponentStatsGroup[];
}
