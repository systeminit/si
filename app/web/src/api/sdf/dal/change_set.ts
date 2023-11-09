// TODO: remove
import { ActionInstance } from "@/store/actions.store";
import { UserId } from "@/store/auth.store";

export enum ChangeSetStatus {
  Open = "Open",
  Applied = "Applied",
  Failed = "Failed",
  Closed = "Closed",
  Abandoned = "Abandoned",
}

export type ChangeSetId = string;
export interface ChangeSet {
  id: ChangeSetId;
  pk: ChangeSetId;
  name: string;
  actions: ActionInstance[];
  status: ChangeSetStatus;
  appliedByUserId?: UserId;
  appliedAt?: IsoDateString;
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
