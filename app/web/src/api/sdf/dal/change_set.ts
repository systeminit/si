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

export interface ComponentStatsGroup {
  component_id: number;
  component_name: string;
}

export interface ComponentStats {
  added: ComponentStatsGroup[];
  deleted: ComponentStatsGroup[];
  modified: ComponentStatsGroup[];
}
