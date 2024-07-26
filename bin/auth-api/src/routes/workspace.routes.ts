import _ from "lodash";
import { nanoid } from "nanoid";
import { z } from "zod";
import { InstanceEnvType } from "@prisma/client";
import { ApiError } from "../lib/api-error";
import { getCache, setCache } from "../lib/cache";
import {
  getUserByEmail,
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
} from "../services/workspaces.service";
import { validate } from "../lib/validation-helpers";

import { CustomRouteContext } from "../custom-state";
import { createSdfAuthToken } from "../services/auth.service";
import { router } from ".";

router.get("/workspaces", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }
  ctx.body = await getUserWorkspaces(ctx.state.authUser.id);
});

// :workspaceId named param handler - little easier for TS this way than using router.param
async function handleWorkspaceIdParam(ctx: CustomRouteContext) {
  if (!ctx.params.workspaceId) {
    throw new Error(
      "Only use this fn with routes containing :workspaceId param",
    );
  }

  // ensure user is logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  // find workspace by id
  const workspace = await getWorkspaceById(ctx.params.workspaceId);
  if (!workspace) {
    throw new ApiError("NotFound", "Workspace not found");
  }

  const memberRole = await userRoleForWorkspace(
    ctx.state.authUser.id,
    workspace.id,
  );
  if (!memberRole) {
    throw new ApiError("Forbidden", "You do not have access to that workspace");
  }

  return workspace;
}

router.get("/workspaces/:workspaceId", async (ctx) => {
  const workspace = await handleWorkspaceIdParam(ctx);
  ctx.body = workspace;
});

router.delete("/workspaces/:workspaceId", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  const workspace = await handleWorkspaceIdParam(ctx);

  await deleteWorkspace(workspace.id);

  ctx.body = "";
});

router.post("/workspaces/setup-production-workspace", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  if (!ctx.state.authUser.email.includes("@systeminit.com")) {
    throw new ApiError(
      "Forbidden",
      "You are not allowed to perform this operation",
    );
  }

  const reqBody = validate(
    ctx.request.body,
    z.object({
      userEmail: z.string(),
    }),
  );

  const user = await getUserByEmail(reqBody.userEmail);
  if (user) {
    const userWorkspaces = await getUserWorkspaces(user.id);
    const hasDefaultWorkspace = _.head(
      _.filter(
        userWorkspaces,
        (w) => w.isDefault && w.creatorUserId === user.id,
      ),
    );

    const workspaceDetails = await createWorkspace(
      user,
      InstanceEnvType.SI,
      "https://app.systeminit.com",
      `${user.nickname}'s Production Workspace`,
      hasDefaultWorkspace === null || hasDefaultWorkspace === undefined,
    );

    ctx.body = {
      newWorkspace: workspaceDetails,
    };
  }
});

router.post("/workspaces/setup-production-workspace-by-userid", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  if (!ctx.state.authUser.email.includes("@systeminit.com")) {
    throw new ApiError(
      "Forbidden",
      "You are not allowed to perform this operation",
    );
  }

  const reqBody = validate(
    ctx.request.body,
    z.object({
      userId: z.string(),
    }),
  );

  const user = await getUserById(reqBody.userId);
  if (user) {
    const userWorkspaces = await getUserWorkspaces(user.id);
    const hasDefaultWorkspace = _.head(
      _.filter(
        userWorkspaces,
        (w) => w.isDefault && w.creatorUserId === user.id,
      ),
    );

    const workspaceDetails = await createWorkspace(
      user,
      InstanceEnvType.SI,
      "https://app.systeminit.com",
      `${user.nickname}'s Production Workspace`,
      hasDefaultWorkspace === null || hasDefaultWorkspace === undefined,
    );

    ctx.body = {
      newWorkspace: workspaceDetails,
    };
  }
});

router.post("/workspaces/new", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  const reqBody = validate(
    ctx.request.body,
    z.object({
      instanceUrl: z.string().url(),
      displayName: z.string(),
      isDefault: z.boolean(),
    }),
  );

  let workspaceEnvType;
  if (reqBody.instanceUrl === "https://app.systeminit.com") {
    workspaceEnvType = InstanceEnvType.SI;
  } else if (reqBody.instanceUrl === "localhost:8080") {
    workspaceEnvType = InstanceEnvType.LOCAL;
  } else {
    workspaceEnvType = InstanceEnvType.PRIVATE;
  }

  const workspaceDetails = await createWorkspace(
    ctx.state.authUser,
    workspaceEnvType,
    reqBody.instanceUrl,
    reqBody.displayName,
    reqBody.isDefault,
  );

  ctx.body = {
    workspaces: await getUserWorkspaces(ctx.state.authUser.id),
    newWorkspaceId: workspaceDetails.id,
  };
});

router.patch("/workspaces/:workspaceId", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  const workspace = await handleWorkspaceIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      instanceUrl: z.string().url(),
      displayName: z.string(),
    }),
  );

  await patchWorkspace(workspace.id, reqBody.instanceUrl, reqBody.displayName);

  ctx.body = await getUserWorkspaces(ctx.state.authUser.id);
});

export type Member = {
  userId: string;
  email: string;
  nickname: string;
  role: string;
  signupAt: Date | null;
};
router.get("/workspace/:workspaceId/members", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  const workspace = await handleWorkspaceIdParam(ctx);

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
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  const workspace = await handleWorkspaceIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      email: z.string(),
    }),
  );

  await inviteMember(reqBody.email, workspace.id);

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

router.delete("/workspace/:workspaceId/members", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  const workspace = await handleWorkspaceIdParam(ctx);

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

  ctx.body = members;
});

router.get("/workspaces/:workspaceId/go", async (ctx) => {
  const workspace = await handleWorkspaceIdParam(ctx);
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const authUser = ctx.state.authUser!;

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

  const token = await createSdfAuthToken(user.id, workspace.id);

  ctx.body = {
    user,
    workspace,
    token,
  };
});

router.get("/auth-reconnect", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You must be logged in");
  }
  if (!ctx.state.authWorkspace) {
    throw new ApiError(
      "Unauthorized",
      "You must pass a workspace-scoped auth token to use this endpoint",
    );
  }

  ctx.body = {
    user: ctx.state.authUser,
    workspace: ctx.state.authWorkspace,
  };
});
