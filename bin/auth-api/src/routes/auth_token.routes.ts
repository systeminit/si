import { z } from "zod";
import { RoleType } from "@prisma/client";
import ms, { StringValue } from "ms";
import { ulid } from "ulidx";
import { validate, ALLOWED_INPUT_REGEX } from "../lib/validation-helpers";

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
import { tracker } from "../lib/tracker";
import { getUserById } from "../services/users.service";
import { automationApiRouter, router } from ".";

// get all authTokens for the given workspace
router.get("/workspaces/:workspaceId/authTokens", async (ctx) => {
  const { workspaceId } = await authorizeWorkspaceRoute(ctx, []);

  const authTokens = await getAuthTokens(workspaceId);

  ctx.body = { authTokens };
});

// create a new authToken for the given workspace
automationApiRouter.post("/workspaces/:workspaceId/authTokens", async (ctx) => {
  const {
    authUser,
    userId,
    workspace,
  } = await authorizeWorkspaceRoute(ctx, [RoleType.OWNER, RoleType.APPROVER, RoleType.EDITOR]);

  // TODO - this should also get an expiration instead of just defaulting to 1d!
  // Get params from body
  const { name, expiration } = validate(
    ctx.request.body,
    z.object({
      name: z.optional(z.string().regex(ALLOWED_INPUT_REGEX)),
      expiration: z.optional(z.string().regex(ALLOWED_INPUT_REGEX)),
    }),
  );

  // Validate expiration
  // (You can pass any string to JWT.sign, but it will treat badly formatter values as "never
  // expire," which is not what we want)
  let expirationStr = expiration?.trim() as StringValue;
  if (!expiration || expiration?.toLocaleLowerCase() === "never" || expiration?.toLocaleLowerCase() === "0") {
    expirationStr = "30d";
  }
  const expiresIn = ms(expirationStr) / 1000;
  if (expiresIn <= 0) throw new ApiError("BadRequest", "zero and negative values not allowed for expiration");
  if (!expiresIn) throw new ApiError("BadRequest", "Invalid expiration format");

  // Create the token
  const token = createSdfAuthToken({
    userId,
    workspaceId: workspace.id,
    role: "automation",
  }, {
    expiresIn,
    jwtid: ulid(),
  });

  // And decode it to get the generated values (such as expiration)
  const authToken = await registerAuthToken(name, await decodeSdfAuthToken(token));

  const workspaceOwner = await getUserById(workspace.creatorUserId);

  tracker.trackEvent(authUser, "workspace_api_token_created", {
    workspaceId: workspace.id,
    workspaceName: workspace.displayName,
    workspaceOwner: workspaceOwner?.email,
    tokenName: name,
    tokenCreated: authToken.createdAt,
    tokenExpires: authToken.expiresAt,
    initiatedBy: authUser.email,
    tokenAction: "created",
  });

  ctx.body = { authToken, token };
});

// get the given authToken for the given workspace
router.get("/workspaces/:workspaceId/authTokens/:authTokenId", async (ctx) => {
  const { authToken } = await authorizeAuthTokenRoute(ctx, []);
  ctx.body = { authToken };
});

// revoke the given authToken for the given workspace
router.post("/workspaces/:workspaceId/authTokens/:authTokenId/revoke", async (ctx) => {
  const { authToken, authUser, workspace } = await authorizeAuthTokenRoute(ctx, [RoleType.OWNER]);

  await updateAuthToken(authToken.id, { revokedAt: new Date() });

  const workspaceOwner = await getUserById(workspace.creatorUserId);

  tracker.trackEvent(authUser, "workspace_api_token_revoked", {
    workspaceId: workspace.id,
    workspaceName: workspace.displayName,
    workspaceOwner: workspaceOwner?.email,
    tokenName: authToken.name,
    tokenCreated: authToken.createdAt,
    tokenRevoked: new Date(),
    initiatedBy: authUser.email,
    reason: "User Initiatiated Action",
    tokenAction: "revoked",
  });

  ctx.body = { authToken };
});

// delete the given authToken for the given workspace
router.delete("/workspaces/:workspaceId/authTokens/:authTokenId", async (ctx) => {
  const {
    authTokenId, authToken, authUser, workspace,
  } = await authorizeAuthTokenRoute(ctx, [RoleType.OWNER]);

  const removed = await deleteAuthToken(authTokenId);

  tracker.trackEvent(authUser, "workspace_api_token_deleted", {
    workspaceId: workspace.id,
    workspaceName: workspace.displayName,
    tokenName: authToken.name,
    tokenCreated: authToken.createdAt,
    tokenDeleted: new Date(),
    initiatedAt: authUser.email,
  });

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
