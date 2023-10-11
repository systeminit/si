import { InstanceEnvType, PrismaClient, User } from '@prisma/client';
import { ulid } from 'ulidx';
import { tracker } from '../lib/tracker';
import { UserId } from "./users.service";

export type WorkspaceId = string;

// this will become a model when we implement db
// export type Workspace = {
//   id: WorkspaceId;
//   instanceType: 'local' | 'private' | 'si_sass'; // only local used for now...
//   instanceUrl: string;
//   displayName: string;
//   // slug: string;
//   // currently workspaces are single player, and controlled by this prop
//   createdByUserId: UserId;
//   createdAt: ISODateTimeString;
// };

const prisma = new PrismaClient();

// TODO: replace all this with actual db calls...
export async function getWorkspaceById(id: WorkspaceId) {
  return await prisma.workspace.findUnique({ where: { id } });
}

export async function createWorkspace(creatorUser: User, instanceUrl = 'http://localhost:8080', displayName = `${creatorUser.nickname}'s dev workspace`) {
  const newWorkspace = await prisma.workspace.create({
    data: {
      id: ulid(),
      instanceEnvType: InstanceEnvType.LOCAL,
      instanceUrl,
      displayName,
      creatorUserId: creatorUser.id,
    },
  });
  tracker.trackEvent(creatorUser, 'create_workspace', {
    workspaceId: newWorkspace.id,
    // TODO: track env type and other data when it becomes useful
  });

  return newWorkspace;
}

export async function patchWorkspace(id: WorkspaceId, instanceUrl: string, displayName: string) {
  return await prisma.workspace.update({ where: { id }, data: { instanceUrl, displayName } });
}

export async function getUserWorkspaces(userId: UserId) {
  const workspaces = await prisma.workspace.findMany({
    where: {
      creatorUserId: userId,
    },
  });
  return workspaces;
}
