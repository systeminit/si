import { nanoid } from "nanoid";
import { z } from 'zod';
import { ApiError } from "../lib/api-error";
import { getCache, setCache } from "../lib/cache";
import { getUserById } from "../services/users.service";
import {
  createWorkspace,
  getUserWorkspaces, getWorkspaceById, patchWorkspace,
} from "../services/workspaces.service";
import { validate } from "../lib/validation-helpers";

import { CustomRouteContext } from "../custom-state";
import { createSdfAuthToken } from "../services/auth.service";
import { router } from ".";

router.get("/workspaces", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }
  ctx.body = await getUserWorkspaces(ctx.state.authUser.id);
});

// :workspaceId named param handler - little easier for TS this way than using router.param
async function handleWorkspaceIdParam(ctx: CustomRouteContext) {
  if (!ctx.params.workspaceId) {
    throw new Error('Only use this fn with routes containing :workspaceId param');
  }

  // ensure user is logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }

  // find workspace by id
  const workspace = await getWorkspaceById(ctx.params.workspaceId);
  if (!workspace) {
    throw new ApiError('NotFound', 'Workspace not found');
  }

  // TODO(Wendy) - here is where we can change which users are allowed to access which workspaces!
  if (workspace.creatorUserId !== ctx.state.authUser.id) {
    throw new ApiError('Forbidden', 'You do not have access to that workspace');
  }

  return workspace;
}

router.get("/workspaces/:workspaceId", async (ctx) => {
  const workspace = await handleWorkspaceIdParam(ctx);
  ctx.body = workspace;
});

router.post("/workspaces/new", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }

  const reqBody = validate(ctx.request.body, z.object({
    instanceUrl: z.string().url(),
    displayName: z.string(),
  }));

  await createWorkspace(ctx.state.authUser, reqBody.instanceUrl, reqBody.displayName);

  ctx.body = await getUserWorkspaces(ctx.state.authUser.id);
});

router.patch("/workspaces/:workspaceId", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }

  const workspace = await handleWorkspaceIdParam(ctx);

  const reqBody = validate(ctx.request.body, z.object({
    instanceUrl: z.string().url(),
    displayName: z.string(),
  }));

  await patchWorkspace(workspace.id, reqBody.instanceUrl, reqBody.displayName);

  ctx.body = await getUserWorkspaces(ctx.state.authUser.id);
});

router.get("/workspaces/:workspaceId/go", async (ctx) => {
  const workspace = await handleWorkspaceIdParam(ctx);
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const authUser = ctx.state.authUser!;

  // generate a new single use authentication code that we will send to the instance
  const connectCode = nanoid(24);
  await setCache(`auth:connect:${connectCode}`, {
    workspaceId: workspace.id,
    userId: authUser.id,
  }, { expiresIn: 60 });

  // redirect to instance (frontend) with single use auth code
  ctx.redirect(`${workspace.instanceUrl}/auth-connect?code=${connectCode}`);
});

router.post("/complete-auth-connect", async (ctx) => {
  const reqBody = validate(ctx.request.body, z.object({
    code: z.string(),
  }));

  const connectPayload = await getCache(`auth:connect:${reqBody.code}`, true);
  if (!connectPayload) throw new ApiError('Conflict', 'Invalid authentication code');

  const workspace = await getWorkspaceById(connectPayload.workspaceId);
  if (!workspace) throw new ApiError('Conflict', 'Workspace no longer exists');

  const user = await getUserById(connectPayload.userId);
  if (!user) throw new ApiError('Conflict', 'User no longer exists');

  const token = await createSdfAuthToken(user.id, workspace.id);

  ctx.body = {
    user,
    workspace,
    token,
  };
});

router.get("/auth-reconnect", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', 'You must be logged in');
  }
  if (!ctx.state.authWorkspace) {
    throw new ApiError('Unauthorized', 'You must pass a workspace-scoped auth token to use this endpoint');
  }

  ctx.body = {
    user: ctx.state.authUser,
    workspace: ctx.state.authWorkspace,
  };
});
