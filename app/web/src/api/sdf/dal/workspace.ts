import { ChangeSetId, ChangeSet } from "./change_set";

export type WorkspacePk = string;

export interface Workspace {
  pk: WorkspacePk;
  name: string;
  created_at: IsoDateString;
  updated_at: IsoDateString;
  token: string;
}

export interface WorkspaceMetadata {
  name: string;
  id: WorkspacePk;
  defaultChangeSetId: ChangeSetId;
  changeSets: ChangeSet[];
  approvers: string[];
}
