import _ from "lodash";
import {
  InstanceEnvType, PrismaClient, User, RoleType, Workspace,
} from "@prisma/client";
import { ulid } from "ulidx";
import { tracker } from "../lib/tracker";
import {
  createInvitedUser,
  getUserByEmail,
  getUserById,
  UserId,
} from "./users.service";

export type WorkspaceId = string;
export const LOCAL_WORKSPACE_URL = "http://localhost:8080";
export const SAAS_WORKSPACE_URL = "https://app.systeminit.com";

const prisma = new PrismaClient();

export async function getWorkspaceById(id: WorkspaceId) {
  return prisma.workspace.findUnique({ where: { id } });
}

export async function createWorkspace(
  creatorUser: User,
  workspaceEnvType: InstanceEnvType,
  instanceUrl: string,
  displayName: string,
  isDefault: boolean,
  description: string,
) {
  const newWorkspace = await prisma.workspace.create({
    data: {
      id: ulid(),
      token: ulid(),
      instanceEnvType: workspaceEnvType,
      instanceUrl,
      displayName,
      creatorUserId: creatorUser.id,
      isDefault,
      description,
    },
  });
  tracker.trackEvent(creatorUser, "create_workspace", {
    workspaceId: newWorkspace.id,
    instanceUrl,
    instanceEnvType: newWorkspace.instanceEnvType,
    isDefaultWorkspace: newWorkspace.isDefault,
  });

  await prisma.workspaceMembers.create({
    data: {
      id: ulid(),
      workspaceId: newWorkspace.id,
      userId: creatorUser.id,
      roleType: RoleType.OWNER,
      invitedAt: new Date(),
    },
  });

  return newWorkspace;
}

export async function deleteWorkspace(id: WorkspaceId) {
  const deletedAt = new Date();
  await prisma.workspace.update({ where: { id }, data: { deletedAt } });
}

export async function patchWorkspace(
  id: WorkspaceId,
  // instanceUrl should never be null, but the prisma type allows it to be so we reproduce this here for now
  instanceUrl: string | null,
  displayName: string,
  quarantinedAt: Date | null,
  description: string | null,
  isFavourite: boolean,
) {
  return prisma.workspace.update({
    where: { id },
    data: {
      instanceUrl: instanceUrl ?? LOCAL_WORKSPACE_URL,
      displayName,
      quarantinedAt,
      description,
      isFavourite,
    },
  });
}

export async function getUserWorkspaces(userId: UserId) {
  const workspaces = await prisma.workspace.findMany({
    where: {
      deletedAt: null,
      quarantinedAt: null,
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
          invitedAt: true,
        },
        where: {
          userId,
        },
      },
      creatorUser: {
        select: {
          firstName: true,
          lastName: true,
        },
      },
    },
  });

  return _.map(workspaces, (w) => ({
    ..._.omit(w, "UserMemberships"),
    role: w.UserMemberships[0].roleType,
    invitedAt: w.UserMemberships[0].invitedAt,
  }));
}

export async function userRoleForWorkspace(
  userId: UserId,
  workspaceId: WorkspaceId,
) {
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

export async function changeWorkspaceMembership(
  workspaceId: WorkspaceId,
  userId: UserId,
  role: string,
) {
  await prisma.workspaceMembers.update({
    where: {
      userId_workspaceId: {
        userId,
        workspaceId,
      },
    },
    data: {
      roleType: roleTypeMap[role],
    },
  });
}

const roleTypeMap: { [key: string]: RoleType } = {
  OWNER: RoleType.OWNER,
  APPROVER: RoleType.APPROVER,
  EDITOR: RoleType.EDITOR,
};

export async function inviteMember(
  authUser: User,
  email: string,
  workspace: Workspace,
) {
  let user = await getUserByEmail(email);
  if (!user) {
    user = await createInvitedUser(email);
    tracker.trackEvent(authUser, "new_user_created_from_invite", {
      workspaceId: workspace.id,
      newUserEmail: email,
      triggeredBy: authUser.email,
      triggeredAt: new Date(),
    });

    // TODO: Paul
    // This will be cleaned up when we have deployed the new transactional emails
    tracker.trackEvent(authUser, "workspace_new_user_invited", {
      workspaceId: workspace.id,
      memberAdded: email,
      memberAddedAt: new Date(),
      invitedBy: authUser.email,
    });
    tracker.trackEvent(authUser, "workspace_new_user_invited_v2", {
      workspaceId: workspace.id,
      workspaceName: workspace.displayName,
      memberUserName: email,
      memberChangedAt: new Date(),
      initiatedBy: authUser.email,
      newPermissionLevel: "Collaborator",
    });
  } else {
    // TODO: Paul
    // This will be cleaned up when we have deployed the new transactional emails
    tracker.trackEvent(authUser, "workspace_existing_user_invited", {
      workspaceId: workspace.id,
      memberAdded: email,
      memberAddedAt: new Date(),
      invitedBy: authUser.email,
    });
    tracker.trackEvent(authUser, "workspace_existing_user_invited_v2", {
      workspaceId: workspace.id,
      workspaceName: workspace.displayName,
      memberUserName: email,
      memberChangedAt: new Date(),
      initiatedBy: authUser.email,
      newPermissionLevel: "Collaborator",
    });
  }

  return await prisma.workspaceMembers.create({
    data: {
      id: ulid(),
      workspaceId: workspace.id,
      userId: user.id,
      roleType: RoleType.EDITOR,
      invitedAt: new Date(),
    },
  });
}

export async function removeUser(email: string, workspaceId: WorkspaceId) {
  const user = await getUserByEmail(email);
  if (!user) {
    return;
  }

  const memberShip = await prisma.workspaceMembers.findFirst({
    where: {
      userId: user.id,
      workspaceId,
    },
  });
  if (!memberShip) {
    return;
  }

  return await prisma.workspaceMembers.delete({
    where: {
      id: memberShip.id,
    },
  });
}

export async function createProductionWorkspaceForUser(userId: UserId) {
  const user = await getUserById(userId);
  if (user) {
    const userWorkspaces = await getUserWorkspaces(user.id);
    const hasDefaultWorkspace = _.head(
      _.filter(
        userWorkspaces,
        (w) => w.isDefault && w.creatorUserId === user.id,
      ),
    );

    if (!hasDefaultWorkspace) {
      const workspaceDetails = await createWorkspace(
        user,
        InstanceEnvType.SI,
        SAAS_WORKSPACE_URL,
        `${user.nickname}'s Production Workspace`,
        hasDefaultWorkspace === null || hasDefaultWorkspace === undefined,
        "",
      );

      return workspaceDetails;
    }
  }

  return null;
}

async function resetDefaultWorkspaces(userId: UserId) {
  await prisma.workspace.updateMany({
    where: {
      deletedAt: null,
      creatorUserId: userId,
    },
    data: {
      isDefault: false,
    },
  });
}

async function setDefaultWorkspace(workspaceId: WorkspaceId) {
  await prisma.workspace.update({
    where: {
      id: workspaceId,
    },
    data: {
      isDefault: true,
    },
  });
}

export async function setUpdatedDefaultWorkspace(
  userId: UserId,
  workspaceId: WorkspaceId,
) {
  await resetDefaultWorkspaces(userId);
  await setDefaultWorkspace(workspaceId);
}
