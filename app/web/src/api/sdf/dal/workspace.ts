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
  defaultChangeSetId: ChangeSetId;
  changeSets: ChangeSet[];
  approvers: string[];
}
