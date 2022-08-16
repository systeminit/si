import { StandardModelNoVisibility } from "@/api/sdf/dal/standard_model";

export enum ChangeSetStatus {
  Open = "Open",
  Closed = "Closed",
  Abandoned = "Abandoned",
  Applied = "Applied",
  Failed = "Failed,",
}

export interface ChangeSet extends StandardModelNoVisibility {
  name: string;
  note?: string;
  status: ChangeSetStatus;
}

export type ComponentStatus = "added" | "deleted" | "modified";

export interface ComponentStatsGroup {
  componentId: number;
  componentName: string;
  componentStatus: ComponentStatus;
}

export interface ComponentStats {
  stats: ComponentStatsGroup[];
}
