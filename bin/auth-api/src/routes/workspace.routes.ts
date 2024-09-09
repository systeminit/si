import { nanoid } from "nanoid";
import { z } from "zod";
import { InstanceEnvType } from "@prisma/client";
import { ApiError } from "../lib/api-error";
import { getCache, setCache } from "../lib/cache";
import {
  getUserById,
  refreshUserAuth0Profile,
} from "../services/users.service";
import {
  createWorkspace,
  getUserWorkspaces,
  getWorkspaceById,
  getWorkspaceMembers,
  inviteMember,
  patchWorkspace,
  deleteWorkspace,
  removeUser,
  userRoleForWorkspace,
  LOCAL_WORKSPACE_URL,
  SAAS_WORKSPACE_URL,
  changeWorkspaceMembership,
} from "../services/workspaces.service";
import { validate } from "../lib/validation-helpers";

import { CustomRouteContext } from "../custom-state";
import { createSdfAuthToken } from "../services/auth.service";
import { tracker } from "../lib/tracker";
import { extractAuthUser, router } from ".";

router.get("/workspaces", async (ctx) => {
  const authUser = extractAuthUser(ctx);
  ctx.body = await getUserWorkspaces(authUser.id);
});

// :workspaceId named param handler - little easier for TS this way than using router.param
export async function extractWorkspaceIdParam(ctx: CustomRouteContext) {
  if (!ctx.params.workspaceId) {
    throw new Error(
      "Only use this fn with routes containing :workspaceId param",
    );
  }

  // find workspace by id
  const workspace = await getWorkspaceById(ctx.params.workspaceId);
  if (!workspace) {
    throw new ApiError("NotFound", "Workspace not found");
  }

  return workspace;
}

// Get a workspace from the param, error if the auth user does not have permission on it
async function extractOwnWorkspaceIdParam(ctx: CustomRouteContext) {
  const workspace = await extractWorkspaceIdParam(ctx);

  const authUser = extractAuthUser(ctx);
  const memberRole = await userRoleForWorkspace(authUser.id, workspace.id);
  if (!memberRole) {
    throw new ApiError("Forbidden", "You do not have access to that workspace");
  }

  return workspace;
}

router.get("/workspaces/:workspaceId", async (ctx) => {
  ctx.body = await extractOwnWorkspaceIdParam(ctx);
});

router.delete("/workspaces/:workspaceId", async (ctx) => {
  const authUser = extractAuthUser(ctx);
  const workspace = await extractOwnWorkspaceIdParam(ctx);

  await deleteWorkspace(workspace.id);

  tracker.trackEvent(authUser, "workspace_deleted", {
    workspaceId: workspace.id,
    workspaceDeletedAt: new Date(),
    workspaceDeletedBy: authUser.email,
  });

  ctx.body = "";
});

router.post("/workspaces/new", async (ctx) => {
  const authUser = extractAuthUser(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      instanceUrl: z.string().url(),
      displayName: z.string(),
      isDefault: z.boolean(),
    }),
  );

  let workspaceEnvType;
  if (reqBody.instanceUrl === SAAS_WORKSPACE_URL) {
    workspaceEnvType = InstanceEnvType.SI;
  } else if (reqBody.instanceUrl === LOCAL_WORKSPACE_URL) {
    workspaceEnvType = InstanceEnvType.LOCAL;
  } else {
    workspaceEnvType = InstanceEnvType.PRIVATE;
  }

  const workspaceDetails = await createWorkspace(
    authUser,
    workspaceEnvType,
    reqBody.instanceUrl,
    reqBody.displayName,
    reqBody.isDefault,
  );

  ctx.body = {
    workspaces: await getUserWorkspaces(authUser.id),
    newWorkspaceId: workspaceDetails.id,
  };
});

router.patch("/workspaces/:workspaceId", async (ctx) => {
  const authUser = extractAuthUser(ctx);

  const workspace = await extractOwnWorkspaceIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      instanceUrl: z.string().url(),
      displayName: z.string(),
    }),
  );

  await patchWorkspace(
    workspace.id,
    reqBody.instanceUrl,
    reqBody.displayName,
    workspace.quarantinedAt,
  );

  tracker.trackEvent(authUser, "workspace_updated", {
    workspaceId: workspace.id,
    workspaceUpdatedAt: new Date(),
    workspaceUpdatedBy: authUser.email,
  });

  ctx.body = await getUserWorkspaces(authUser.id);
});

export type Member = {
  userId: string;
  email: string;
  nickname: string;
  role: string;
  signupAt: Date | null;
};
router.get("/workspace/:workspaceId/members", async (ctx) => {
  extractAuthUser(ctx);

  const workspace = await extractOwnWorkspaceIdParam(ctx);

  const members: Member[] = [];
  const workspaceMembers = await getWorkspaceMembers(workspace.id);

  workspaceMembers.forEach((wm) => {
    members.push({
      userId: wm.userId,
      email: wm.user.email,
      nickname: wm.user.nickname || "",
      role: wm.roleType,
      signupAt: wm.user.signupAt,
    });
  });

  ctx.body = members;
});

router.post("/workspace/:workspaceId/membership", async (ctx) => {
  // user must be logged in
  const authUser = extractAuthUser(ctx);

  const workspace = await extractOwnWorkspaceIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      userId: z.string(),
      role: z.string(),
    }),
  );

  tracker.trackEvent(authUser, "workspace_membership_roles_changed", {
    role: reqBody.role,
    userId: reqBody.userId,
    workspaceId: workspace.id,
  });

  await changeWorkspaceMembership(workspace.id, reqBody.userId, reqBody.role);

  const members: Member[] = [];
  const workspaceMembers = await getWorkspaceMembers(workspace.id);

  workspaceMembers.forEach((wm) => {
    members.push({
      userId: wm.userId,
      email: wm.user.email,
      nickname: wm.user.nickname || "",
      role: wm.roleType,
      signupAt: wm.user.signupAt,
    });
  });

  ctx.body = members;
});

router.post("/workspace/:workspaceId/members", async (ctx) => {
  // user must be logged in
  const authUser = extractAuthUser(ctx);

  const workspace = await extractOwnWorkspaceIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      email: z.string(),
    }),
  );

  await inviteMember(authUser, reqBody.email, workspace.id);

  const members: Member[] = [];
  const workspaceMembers = await getWorkspaceMembers(workspace.id);

  workspaceMembers.forEach((wm) => {
    members.push({
      userId: wm.userId,
      email: wm.user.email,
      nickname: wm.user.nickname || "",
      role: wm.roleType,
      signupAt: wm.user.signupAt,
    });
  });

  tracker.trackEvent(authUser, "workspace_user_invited", {
    workspaceId: workspace.id,
    memberAdded: reqBody.email,
    memberAddedAt: new Date(),
  });

  ctx.body = members;
});

router.delete("/workspace/:workspaceId/members", async (ctx) => {
  // user must be logged in
  const authUser = extractAuthUser(ctx);

  const workspace = await extractOwnWorkspaceIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      email: z.string(),
    }),
  );

  await removeUser(reqBody.email, workspace.id);

  const members: Member[] = [];
  const workspaceMembers = await getWorkspaceMembers(workspace.id);
  workspaceMembers.forEach((wm) => {
    members.push({
      userId: wm.userId,
      email: wm.user.email,
      nickname: wm.user.nickname || "",
      role: wm.roleType,
      signupAt: wm.user.signupAt,
    });
  });

  tracker.trackEvent(authUser, "workspace_user_removed", {
    workspaceId: workspace.id,
    memberRemoved: reqBody.email,
    memberRemovedAt: new Date(),
  });

  ctx.body = members;
});

router.get("/workspaces/:workspaceId/go", async (ctx) => {
  const workspace = await extractOwnWorkspaceIdParam(ctx);

  if (workspace.quarantinedAt !== null) {
    throw new ApiError(
      "Unauthorized",
      `This workspace (ID ${workspace.id}) is quarantined. Contact SI support`,
    );
  }

  const authUser = extractAuthUser(ctx);

  // we require the user to have verified their email before they can log into a workspace
  if (!authUser.emailVerified) {
    // we'll first refresh from auth0 to make sure its actually not verified
    await refreshUserAuth0Profile(authUser);
    // then throw an error
    if (!authUser.emailVerified) {
      throw new ApiError(
        "Unauthorized",
        "EmailNotVerified",
        "System Initiative Requires Verified Emails to access Workspaces. Check your registered email for Verification email from SI Auth Portal.",
      );
    }
  }

  // generate a new single use authentication code that we will send to the instance
  const connectCode = nanoid(24);
  await setCache(
    `auth:connect:${connectCode}`,
    {
      workspaceId: workspace.id,
      userId: authUser.id,
    },
    { expiresIn: 60 },
  );

  // redirect to instance (frontend) with single use auth code
  ctx.redirect(`${workspace.instanceUrl}/auth-connect?code=${connectCode}`);
});

router.post("/complete-auth-connect", async (ctx) => {
  const reqBody = validate(
    ctx.request.body,
    z.object({
      code: z.string(),
    }),
  );

  const connectPayload = await getCache(`auth:connect:${reqBody.code}`, true);
  if (!connectPayload) throw new ApiError("Conflict", "Invalid authentication code");

  const workspace = await getWorkspaceById(connectPayload.workspaceId);
  if (!workspace) throw new ApiError("Conflict", "Workspace no longer exists");

  const user = await getUserById(connectPayload.userId);
  if (!user) throw new ApiError("Conflict", "User no longer exists");

  const token = createSdfAuthToken(user.id, workspace.id);

  ctx.body = {
    user,
    workspace,
    token,
  };
});

router.get("/auth-reconnect", async (ctx) => {
  const authUser = extractAuthUser(ctx);
  if (!ctx.state.authWorkspace) {
    throw new ApiError(
      "Unauthorized",
      "You must pass a workspace-scoped auth token to use this endpoint",
    );
  }

  ctx.body = {
    user: authUser,
    workspace: ctx.state.authWorkspace,
  };
});
