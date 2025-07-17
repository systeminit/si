import { UserId } from "@/store/auth.store";

export enum ChangeSetStatus {
  Open = "Open",
  Applied = "Applied",
  Failed = "Failed",
  Closed = "Closed",
  Abandoned = "Abandoned",
  NeedsApproval = "NeedsApproval",
  NeedsAbandonApproval = "NeedsAbandonApproval",
  Rejected = "Rejected",
  Approved = "Approved",
}

export type ChangeSetId = string;
export interface ChangeSet {
  id: ChangeSetId;
  name: string;
  status: ChangeSetStatus;
  appliedByUserId?: UserId;
  appliedAt?: IsoDateString;
  mergeRequestedAt?: IsoDateString;
  mergeRequestedByUserId?: UserId;
  mergeRequestedByUser?: string;
  baseChangeSetId: ChangeSetId | null;
  reviewedByUserId?: UserId;
  reviewedByUser?: string;
  reviewedAt?: IsoDateString;
  updatedAt?: IsoDateString;
  abandonRequestedAt?: IsoDateString;
  abandonRequestedByUserId?: UserId;
  isHead?: boolean;
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
