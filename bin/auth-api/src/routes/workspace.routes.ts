import { nanoid } from "nanoid";
import { z } from 'zod';
import { Workspace } from "@prisma/client";
import { ApiError } from "../lib/api-error";
import { getCache, setCache } from "../lib/cache";
import { getUserById } from "../services/users.service";
import {
  getUserWorkspaces, getWorkspaceById, WorkspaceId,
} from "../services/workspaces.service";
import { validate } from "../lib/validation-helpers";

import { CustomAppState } from "../custom-state";
import { createSdfAuthToken } from "../services/auth.service";
import { router } from ".";

router.get("/workspaces", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }
  ctx.body = await getUserWorkspaces(ctx.state.authUser.id);
});

// our param handler ensures authUser and workspace are set
type WorkspaceRoutesCtxState = {
  workspace: Workspace
} & Required<Pick<CustomAppState, 'authUser'>>;

// named param handler - will fire for all routes with :workspaceId in url
router.param('workspaceId', async (id: WorkspaceId, ctx, next) => {
  // ensure user is logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }

  // TODO: maybe some checking/validation on the id itself

  // find workspace by id
  const workspace = await getWorkspaceById(id);
  if (!workspace) {
    throw new ApiError('NotFound', 'Workspace not found');
  }

  // overly simplified access model...
  if (workspace.creatorUserId !== ctx.state.authUser.id) {
    throw new ApiError('Forbidden', 'You do not have access to that workspace');
  }

  // store workspace on ctx.state for use by other routes
  (ctx.state as any).workspace = workspace;
  return next();
});

router.get<WorkspaceRoutesCtxState>("/workspaces/:workspaceId", async (ctx) => {
  ctx.body = ctx.state.workspace;
});

router.patch<WorkspaceRoutesCtxState>("/workspaces/:workspaceId", async (ctx) => {
  ctx.body = ctx.state.workspace;
});

router.get<WorkspaceRoutesCtxState>("/workspaces/:workspaceId/go", async (ctx) => {
  const { authUser, workspace } = ctx.state;

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

  const connectPayload = await getCache(`auth:connect:${reqBody.code}`);
  if (!connectPayload) throw new ApiError('Forbidden', 'Invalid authentication code');

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

router.get("/test", async (ctx) => {
  const reqQuery = validate(ctx.request.query, z.object({
    code: z.number().optional().transform((v) => {
      if (v === undefined) return undefined;
      if (v < 0) return 0;
      if (v > 100) return 100;
      return Math.round(v);
    }),
  }));

  ctx.body = reqQuery;
});
