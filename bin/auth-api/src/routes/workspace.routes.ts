import { nanoid } from "nanoid";
import { z } from "zod";
import { InstanceEnvType, RoleType } from "@prisma/client";
import { ApiError } from "../lib/api-error";
import { getCache, setCache } from "../lib/cache";
import {
  getUserById,
  refreshUserAuth0Profile,
  UserId,
} from "../services/users.service";
import { revokeAllWorkspaceTokens } from "../services/auth_tokens.service";
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
  setUpdatedDefaultWorkspace,
  WorkspaceId,
} from "../services/workspaces.service";
import {
  validate,
  ALLOWED_INPUT_REGEX,
  ALLOWED_URL_REGEX,
} from "../lib/validation-helpers";

import { CustomRouteContext } from "../custom-state";
import {
  makeAuthConnectUrl,
  createSdfAuthToken,
} from "../services/auth.service";
import { tracker } from "../lib/tracker";
import { findLatestTosForUser } from "../services/tos.service";
import { automationApiRouter, extractAuthUser, router } from ".";

automationApiRouter.get("/workspaces", async (ctx) => {
  const authUser = extractAuthUser(ctx);
  ctx.body = await getUserWorkspaces(authUser.id);
});

/// Extract the workspace data from the request. YOU WANT TO USE authorizeWorkspaceRoute
export async function extractWorkspaceIdParamWithoutAuthorizing(ctx: CustomRouteContext) {
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

// TODO this means that admin do not get automatic access to endpoints that call this
export async function authorizeWorkspaceRoute(
  ctx: CustomRouteContext,
  roles: RoleType[] = [],
) {
  const workspace = await extractWorkspaceIdParamWithoutAuthorizing(ctx);
  const authUser = extractAuthUser(ctx);

  const memberRole = await userRoleForWorkspace(authUser.id, workspace.id);

  if (!memberRole) {
    throw new ApiError("Forbidden", "You are not member of this workspace");
  }

  if (roles.length > 0 && !roles.includes(memberRole)) {
    throw new ApiError(
      "Forbidden",
      `You must be one of the following roles to interact with this workspace: ${roles.join(
        ", ",
      )}`,
    );
  }

  return {
    authUser,
    workspace,
    // Conveniences for destructuring for routes that just need IDs
    userId: authUser.id as UserId,
    workspaceId: workspace.id as WorkspaceId,
  };
}

automationApiRouter.get("/workspaces/:workspaceId", async (ctx) => {
  const { workspace } = await authorizeWorkspaceRoute(ctx);
  ctx.body = workspace;
});

automationApiRouter.delete("/workspaces/:workspaceId", async (ctx) => {
  extractAuthUser(ctx, true);
  const { authUser, workspaceId } = await authorizeWorkspaceRoute(ctx, [RoleType.OWNER]);

  const workspace = await getWorkspaceById(workspaceId);
  if (!workspace) throw new ApiError("Conflict", "Workspace doesn't exist");

  const workspaceOwner = await getUserById(workspace.creatorUserId)!;

  await deleteWorkspace(workspace.id);

  tracker.trackEvent(authUser, "workspace_deleted", {
    workspaceId,
    workspaceDeletedAt: new Date(),
    workspaceDeletedBy: authUser.email,
  });

  const { tokensToRevoke } = await revokeAllWorkspaceTokens(workspaceId);
  tokensToRevoke.forEach((token) => {
    tracker.trackEvent(authUser, "workspace_api_token_revoked", {
      workspaceId: workspace.id,
      workspaceName: workspace.displayName,
      workspaceOwner: workspaceOwner?.email,
      tokenName: token.name,
      tokenCreated: token.createdAt,
      tokenRevoked: new Date(),
      initiatedBy: authUser.email,
      reason: "User deleted workspace",
      tokenAction: "revoked",
    });
  });

  ctx.body = "";
});

automationApiRouter.post("/workspaces/new", async (ctx) => {
  const authUser = extractAuthUser(ctx, true);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      instanceUrl: z.string().url().regex(new RegExp(ALLOWED_URL_REGEX)),
      displayName: z.string().regex(ALLOWED_INPUT_REGEX),
      isDefault: z.boolean(),
      description: z.string().regex(ALLOWED_INPUT_REGEX),
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
    reqBody.description,
  );

  ctx.body = {
    workspaces: await getUserWorkspaces(authUser.id),
    newWorkspaceId: workspaceDetails.id,
  };
});

automationApiRouter.patch("/workspaces/:workspaceId", async (ctx) => {
  extractAuthUser(ctx, true);
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx, [
    RoleType.OWNER,
  ]);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      instanceUrl: z.string().url(),
      displayName: z.string().regex(ALLOWED_INPUT_REGEX),
      description: z.string().regex(ALLOWED_INPUT_REGEX),
    }),
  );

  await patchWorkspace(
    workspace.id,
    reqBody.instanceUrl,
    reqBody.displayName,
    workspace.quarantinedAt,
    reqBody.description,
    workspace.isFavourite,
    workspace.isHidden,
    workspace.approvalsEnabled,
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
automationApiRouter.get("/workspace/:workspaceId/members", async (ctx) => {
  const { workspace } = await authorizeWorkspaceRoute(ctx, undefined);

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

// When we send a hubspot email via the posthog event
// if the workspace name is a domain name like string e.g. bing.com
// then when the email gets sent, it will render as a link to bing.com
// rather than the workspace name as a string
// this adds a zero width space to stop the email clients from rendering it
// it will still look like bing.com but it's effectively breaking the link
function escapeDomainLikeString(input: string): string {
  return input.replace(/\./g, "\u200B.");
}

automationApiRouter.post("/workspace/:workspaceId/membership", async (ctx) => {
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx, [
    RoleType.OWNER,
  ]);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      userId: z.string(),
      role: z.string(),
    }),
  );

  const user = await getUserById(reqBody.userId);

  tracker.trackEvent(authUser, "workspace_membership_roles_changed_v2", {
    newPermissionLevel: reqBody.role === "EDITOR" ? "Collaborator" : "Approver",
    memberUserName: user?.email || "",
    workspaceId: workspace.id,
    workspaceName: escapeDomainLikeString(workspace.displayName),
    initiatedBy: authUser.email,
    memberChangedAt: new Date(),
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

automationApiRouter.post("/workspace/:workspaceId/members", async (ctx) => {
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx, [
    RoleType.OWNER,
    RoleType.APPROVER,
  ]);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      email: z.string().email(),
    }),
  );

  try {
    await inviteMember(authUser, reqBody.email, workspace);
  } catch (error) {
    ctx.status = 409;
    ctx.body = { error: (error as Error).message };
    return;
  }

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

automationApiRouter.delete("/workspace/:workspaceId/members", async (ctx) => {
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx, [
    RoleType.OWNER,
    RoleType.APPROVER,
  ]);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      email: z.string().email(),
    }),
  );

  const workspaceMembers = await getWorkspaceMembers(workspace.id);

  const userToRemove = workspaceMembers.find(
    (wm) => wm.user.email === reqBody.email,
  );

  if (!userToRemove || !userToRemove.user) {
    ctx.status = 404;
    ctx.body = { error: "User not found in workspace" };
    return;
  }

  await removeUser(userToRemove.userId, workspace.id);

  const members: Member[] = workspaceMembers
    .filter((wm) => wm.user.email !== reqBody.email)
    .map((wm) => ({
      userId: wm.userId,
      email: wm.user.email,
      nickname: wm.user.nickname || "",
      role: wm.roleType,
      signupAt: wm.user.signupAt,
    }));

  tracker.trackEvent(authUser, "workspace_user_removed_v2", {
    workspaceId: workspace.id,
    workspaceName: escapeDomainLikeString(workspace.displayName),
    initiatedBy: authUser.email,
    memberUserName: reqBody.email,
    memberChangedAt: new Date(),
    newPermissionLevel: "No Access",
  });

  ctx.body = members;
});

router.patch("/workspaces/:workspaceId/setDefault", async (ctx) => {
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx);

  tracker.trackEvent(authUser, "set_default_workspace", {
    defaultWorkspaceSetBy: authUser.email,
    workspaceId: workspace.id,
  });

  // update all existing workspaces to not be default
  await setUpdatedDefaultWorkspace(authUser.id, workspace.id);

  // Return the updated workspace list
  ctx.body = await getUserWorkspaces(authUser.id);
});

router.patch("/workspaces/:workspaceId/favourite", async (ctx) => {
  extractAuthUser(ctx, true);
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      isFavourite: z.boolean(),
    }),
  );

  const favouriteDate = new Date();
  if (reqBody.isFavourite) {
    tracker.trackEvent(authUser, "favourite_workspace", {
      favouritedBy: authUser.email,
      favouriteDate,
      workspaceId: workspace.id,
    });
  } else {
    tracker.trackEvent(authUser, "unfavourite_workspace", {
      unFavouritedBy: authUser.email,
      unFavouriteDate: favouriteDate,
      workspaceId: workspace.id,
    });
  }

  await patchWorkspace(
    workspace.id,
    workspace.instanceUrl,
    workspace.displayName,
    workspace.quarantinedAt,
    workspace.description,
    reqBody.isFavourite,
    workspace.isHidden,
    workspace.approvalsEnabled,
  );

  ctx.body = await getUserWorkspaces(authUser.id);
});

router.patch("/workspaces/:workspaceId/approvalsEnabled", async (ctx) => {
  extractAuthUser(ctx, true);
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx, [RoleType.OWNER]);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      approvalsEnabled: z.boolean(),
    }),
  );

  const approvalsStatusChangeDate = new Date();
  if (reqBody.approvalsEnabled) {
    tracker.trackEvent(authUser, "enable_workspace_approvals", {
      approvalsEnabledBy: authUser.email,
      approvalsEnabledDate: approvalsStatusChangeDate,
      workspaceId: workspace.id,
    });
  } else {
    tracker.trackEvent(authUser, "disable_workspace_approvals", {
      approvalsDisabledBy: authUser.email,
      approvalsDisabledDate: approvalsStatusChangeDate,
      workspaceId: workspace.id,
    });
  }

  await patchWorkspace(
    workspace.id,
    workspace.instanceUrl,
    workspace.displayName,
    workspace.quarantinedAt,
    workspace.description,
    workspace.isFavourite,
    workspace.isHidden,
    reqBody.approvalsEnabled,
  );

  ctx.body = await getUserWorkspaces(authUser.id);
});

router.patch("/workspaces/:workspaceId/setHidden", async (ctx) => {
  extractAuthUser(ctx, true);
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      isHidden: z.boolean(),
    }),
  );

  const hiddenDate = new Date();
  if (reqBody.isHidden) {
    tracker.trackEvent(authUser, "hide_workspace", {
      hiddenBy: authUser.email,
      hiddenDate,
      workspaceId: workspace.id,
    });
  } else {
    tracker.trackEvent(authUser, "unhide_workspace", {
      unHiddenBy: authUser.email,
      unHiddenDate: hiddenDate,
      workspaceId: workspace.id,
    });
  }

  await patchWorkspace(
    workspace.id,
    workspace.instanceUrl,
    workspace.displayName,
    workspace.quarantinedAt,
    workspace.description,
    workspace.isFavourite,
    reqBody.isHidden,
    workspace.approvalsEnabled,
  );

  ctx.body = await getUserWorkspaces(authUser.id);
});

router.get("/workspaces/:workspaceId/go", async (ctx) => {
  const { authUser, workspace } = await authorizeWorkspaceRoute(ctx);

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

  const latestTos = await findLatestTosForUser(authUser);
  if (latestTos > authUser.agreedTosVersion) {
    throw new ApiError(
      "Unauthorized",
      "MissingTosAcceptance",
      "Terms of Service have been updated, return to the SI auth portal to accept them.",
    );
  }

  const { redirect } = validate(
    ctx.request.query,
    z.object({
      redirect: z.string().optional(),
    }),
  );

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

  const redirectUrl = await makeAuthConnectUrl(workspace, authUser, connectCode, redirect);

  // redirect to instance (frontend) with single use auth code
  ctx.redirect(redirectUrl);
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

  const token = createSdfAuthToken({
    userId: user.id,
    workspaceId: workspace.id,
    role: "web",
  });

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

  const body: {
    user: typeof authUser,
    workspace: typeof ctx.state.authWorkspace
    onDemandAssets?: boolean,
  } = {
    user: authUser,
    workspace: ctx.state.authWorkspace,
  };

  body.onDemandAssets = true;

  ctx.body = { ...body };
});
