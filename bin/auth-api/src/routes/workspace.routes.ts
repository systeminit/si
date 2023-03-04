import _ from "lodash";
import Router from "@koa/router";
import { nanoid } from "nanoid";
import { ulid } from "ulidx";
import { ApiError } from "../lib/api-error";
import { getCache, setCache } from "../lib/cache";
import { createSdfAuthToken } from "../services/auth";
import { getUserById } from "../services/users.service";

const DUMMY_WS_DATA = [
  {
    // id: ulid(),
    id: '01GTZMTEV4T9PAA1TWVWH3ZQR4', // ulid
    instanceType: 'local',
    instanceUrl: 'http://localhost:8080',
    displayName: 'Default dev workspace',
    slug: 'dev1',
    // createdByUserId: ctx.$.user.id,
    createdByUserId: 'usr_sysinit',
    createdAt: '2023-03-06T23:45:49.326Z',
  },
];

export function initRoutes(router: Router) {
  router.get("/workspaces", async (ctx) => {
    // user must be logged in
    if (!ctx.$.user) {
      throw new ApiError('Unauthorized', "You are not logged in");
    }
    ctx.body = {
      // using dummy data for now... will fetch from db
      workspaces: DUMMY_WS_DATA,
    };
  });

  // named param handler - will fire for all routes with :workspaceId in url
  router.param('workspaceId', async (id, ctx, next) => {
    if (!ctx.$.user) {
      throw new ApiError('Unauthorized', "You are not logged in");
    }

    const workspace = _.find(DUMMY_WS_DATA, (w) => w.id === id);
    if (!workspace) {
      throw new ApiError('NotFound', 'Workspace not found');
    }

    // TODO: check basic access to the workspace
    ctx.$.workspace = workspace;
    return next();
  });

  router.get("/workspaces/:workspaceId", async (ctx) => {
    ctx.body = ctx.$.workspace;
  });

  router.get("/workspaces/:workspaceId/go", async (ctx) => {
    const { workspace } = ctx.$;

    // generate a new single use authentication code that we will send to the instance
    const connectCode = nanoid(24);
    await setCache(`auth:connect:${connectCode}`, {
      workspaceId: workspace.id,
      userId: ctx.$.user.id,
    }, { expiresIn: 60 });

    // redirect to instance (frontend) with single use auth code
    ctx.redirect(`${workspace.instanceUrl}/auth-connect?code=${connectCode}`);
  });

  router.post("/complete-auth-connect", async (ctx) => {
    console.log(ctx.request.rawBody, ctx.request.body);
    // TODO: swap in better body / validation tooling
    const connectCode = (ctx.request.body as any).code as string | undefined;
    if (!connectCode) throw new ApiError('BadRequest', 'Missing code');

    const connectPayload = await getCache(`auth:connect:${connectCode}`);
    if (!connectPayload) throw new ApiError('Forbidden', 'Invalid authentication code');

    // TODO: lookup info in DB
    const workspace = _.find(DUMMY_WS_DATA, (w) => w.id === connectPayload.workspaceId);

    const userId = connectPayload.userId;
    const user = await getUserById(userId);

    const token = await createSdfAuthToken(userId);

    ctx.body = {
      user,
      workspace,
      token,
    };
  });
}
