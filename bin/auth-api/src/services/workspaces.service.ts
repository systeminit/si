import _ from "lodash";
import {
  InstanceEnvType, PrismaClient, User, RoleType,
} from '@prisma/client';
import { ulid } from 'ulidx';
import { tracker } from '../lib/tracker';
import { createInvitedUser, getUserByEmail, UserId } from "./users.service";

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

  await prisma.workspaceMembers.create({
    data: {
      id: ulid(),
      workspaceId: newWorkspace.id,
      userId: creatorUser.id,
      roleType: RoleType.OWNER,
    },
  });

  return newWorkspace;
}

export async function patchWorkspace(id: WorkspaceId, instanceUrl: string, displayName: string) {
  return await prisma.workspace.update({ where: { id }, data: { instanceUrl, displayName } });
}

export async function getUserWorkspaces(userId: UserId) {
  const workspaces = await prisma.workspace.findMany({
    where: {
      UserMemberships: {
        some: {
          userId,
        },
      },
    },
    include: {
      UserMemberships: {
        select: {
          roleType: true,
        },
        where: {
          userId,
        },
      },
    },
  });

  return _.map(workspaces, (w) => ({
    ..._.omit(w, "UserMemberships"),
    role: w.UserMemberships[0].roleType,
  }));

  return workspaces;
}

export async function userRoleForWorkspace(userId: UserId, workspaceId: WorkspaceId) {
  const member = await prisma.workspaceMembers.findFirst({
    where: {
      userId,
      workspaceId,
    },
  });

  return member?.roleType;
}

export async function getWorkspaceMembers(id: WorkspaceId) {
  const workspaceMembers = await prisma.workspaceMembers.findMany({
    where: {
      workspaceId: id,
    },
    include: {
      user: true,
    },
  });

  return workspaceMembers;
}

export async function inviteCollaborator(email: string, id: WorkspaceId) {
  let user = await getUserByEmail(email);
  if (!user) {
    user = await createInvitedUser(email);
  }

  return await prisma.workspaceMembers.create({
    data: {
      id: ulid(),
      workspaceId: id,
      userId: user.id,
      roleType: RoleType.COLLABORATOR,
    },
  });
}
