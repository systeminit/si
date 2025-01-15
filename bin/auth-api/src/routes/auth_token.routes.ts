import { z } from "zod";
import { RoleType } from "@prisma/client";
import { ulid } from "ulidx";
import { validate } from "../lib/validation-helpers";

import { CustomRouteContext } from "../custom-state";
import {
  createSdfAuthToken,
  decodeSdfAuthToken,
} from "../services/auth.service";
import {
  AuthTokenId,
  getAuthToken,
  registerAuthToken,
  updateAuthToken,
  deleteAuthToken,
  getAuthTokens,
} from "../services/auth_tokens.service";
import { authorizeWorkspaceRoute } from "./workspace.routes";
import { ApiError } from "../lib/api-error";
import { router } from ".";

// get all authTokens for the given workspace
router.get("/workspaces/:workspaceId/authTokens", async (ctx) => {
  const { workspaceId } = await authorizeWorkspaceRoute(ctx, []);

  const authTokens = await getAuthTokens(workspaceId);

  ctx.body = { authTokens };
});

// create a new authToken for the given workspace
router.post("/workspaces/:workspaceId/authTokens", async (ctx) => {
  const {
    userId,
    workspaceId,
  } = await authorizeWorkspaceRoute(ctx, [RoleType.OWNER]);

  // TODO - this should also get an expiration instead of just defaulting to 1d!
  // Get params from body
  const { name, expiration } = validate(
    ctx.request.body,
    z.object({
      name: z.optional(z.string()),
      expiration: z.optional(z.string()),
    }),
  );

  let expiresIn;
  if (expiration && expiration.trim().toLocaleLowerCase() !== 'never' && expiration.trim().toLocaleLowerCase() !== "0") {
    expiresIn = expiration;
  } else {
    expiresIn = "30d";
  }

  // Create the token
  const token = createSdfAuthToken({
    userId,
    workspaceId,
    role: "automation",
  }, {
    expiresIn,
    jwtid: ulid(),
  });

  // And decode it to get the generated values (such as expiration)
  const authToken = await registerAuthToken(name, await decodeSdfAuthToken(token));

  ctx.body = { authToken, token };
});

// get the given authToken for the given workspace
router.get("/workspaces/:workspaceId/authTokens/:authTokenId", async (ctx) => {
  const { authToken } = await authorizeAuthTokenRoute(ctx, []);
  ctx.body = { authToken };
});

// rename the given authToken for the given workspace
router.put("/workspaces/:workspaceId/authTokens/:authTokenId", async (ctx) => {
  const { authToken } = await authorizeAuthTokenRoute(ctx, [RoleType.OWNER]);

  // Get params from body
  const { name } = validate(
    ctx.request.body,
    z.object({
      name: z.nullable(z.string()),
    }),
  );

  await updateAuthToken(authToken.id, { name });

  ctx.body = { authToken };
});

// revoke the given authToken for the given workspace
router.post("/workspaces/:workspaceId/authTokens/:authTokenId/revoke", async (ctx) => {
  const { authToken } = await authorizeAuthTokenRoute(ctx, [RoleType.OWNER]);

  await updateAuthToken(authToken.id, { revokedAt: new Date() });

  ctx.body = { authToken };
});

// delete the given authToken for the given workspace
router.delete("/workspaces/:workspaceId/authTokens/:authTokenId", async (ctx) => {
  const { authTokenId } = await authorizeAuthTokenRoute(ctx, [RoleType.OWNER]);

  const removed = await deleteAuthToken(authTokenId);

  ctx.body = { removed };
});

// Authorize /workspaces/:workspaceId/authTokens/:authTokenId
// FIXME: APPROVER and EDITOR cannot be explicitly set since it would disallow OWNER
async function authorizeAuthTokenRoute(ctx: CustomRouteContext, roles: RoleType[] = []) {
  if (!ctx.params.authTokenId) {
    throw new Error(`No :authTokenId param on route: ${ctx.params}`);
  }

  const route = await authorizeWorkspaceRoute(ctx, roles);
  const authToken = await getAuthToken(ctx.params.authTokenId);
  if (!authToken) {
    throw new ApiError("NotFound", "AuthToken not found");
  }
  if (authToken.workspaceId !== route.workspaceId) {
    throw new ApiError("Unauthorized", "AuthToken does not belong to workspace");
  }

  // NOTE: we don't check userId of token because we require you to be a workspace
  // owner, which means you can revoke any token in the workspace, regardless of who
  // it grants permission to.
  //
  // Users probably need to be able to delete their own tokens even if they don't own the
  // workspaces, but since only owners can create tokens and you can't transfer ownership,
  // that situation won't happen for now. We can revisit when it does.

  return {
    ...route,
    authTokenId: authToken.id as AuthTokenId,
    authToken,
  };
}
