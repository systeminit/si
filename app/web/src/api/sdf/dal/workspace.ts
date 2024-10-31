import { ChangeSetId, ChangeSet } from "./change_set";

export interface Workspace {
  pk: string;
  name: string;
  created_at: IsoDateString;
  updated_at: IsoDateString;
  token: string;
}

export interface WorkspaceMetadata {
  name: string;
  id: string;
  default_change_set_id: ChangeSetId;
  change_sets: ChangeSet[];
  approvers: string[];
}
