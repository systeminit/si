// TODO: remove
import { ActionInstance } from "@/store/actions.store";

export enum ChangeSetStatus {
  Open = "Open",
  Closed = "Closed",
  Abandoned = "Abandoned",
  Applied = "Applied",
  Failed = "Failed",
}

export type ChangeSetId = string;
export interface ChangeSet {
  id: ChangeSetId;
  pk: ChangeSetId;
  name: string;
  actions: ActionInstance[];
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
