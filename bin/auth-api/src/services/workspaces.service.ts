import _ from 'lodash';
import { ulid } from 'ulidx';
import { getCache, setCache } from "../lib/cache";
import { User, UserId } from "./users.service";

export type WorkspaceId = string;

// this will become a model when we implement db
export type Workspace = {
  id: WorkspaceId;
  instanceType: 'local' | 'private' | 'si_sass'; // only local used for now...
  instanceUrl: string;
  displayName: string;
  slug: string;
  // currently workspaces are single player, and controlled by this prop
  createdByUserId: UserId;
  createdAt: ISODateTimeString;
};

// temporary just store in memory
const workspacesById: Record<WorkspaceId, Workspace> = {};

// TODO: replace all this with actual db calls...
export async function getWorkspaceById(id: WorkspaceId) {
  return _.find(_.values(workspacesById), (w) => w.id === id);
}

export async function createWorkspace(creatorUser: User) {
  const workspace: Workspace = {
    id: ulid(),
    instanceType: 'local',
    instanceUrl: 'http://localhost:8080',
    displayName: `${creatorUser.nickname}'s dev workspace`,
    slug: 'dev',
    createdByUserId: creatorUser.id,
    createdAt: new Date().toISOString(),
  };
  workspacesById[workspace.id] = workspace;
  return workspace;
}
export async function saveWorkspace(workspace: Workspace) {
  workspacesById[workspace.id] = workspace;
}
export async function getUserWorkspaces(userId: UserId) {
  return _.filter(_.values(workspacesById), (w) => w.createdByUserId === userId);
}
