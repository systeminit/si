import { User, Workspace } from "@prisma/client";
import { JwtPayload, SignOptions } from "jsonwebtoken";
import * as Koa from "koa";
import { nanoid } from "nanoid";
import { CustomAppContext, CustomAppState } from "../custom-state";
import { ApiError } from "../lib/api-error";
import { setCache } from "../lib/cache";
import { createJWT, verifyJWT } from "../lib/jwt";
import { tryCatch } from "../lib/try-catch";
import { getUserById, UserId } from "./users.service";
import { getWorkspaceById, WorkspaceId } from "./workspaces.service";
import { getAuthToken } from "./auth_tokens.service";
import { posthog } from "../lib/posthog";

export const SI_COOKIE_NAME = "si-auth";

export type AuthProviders = "google" | "github" | "password";

// TODO: figure out the shape of the JWT and what data we want

// Auth tokens used for communication between the user's browser and this auth api
export interface AuthTokenData {
  userId: string;
  workspaceId?: string;
  role: SdfAuthTokenRole;
  tokenId?: string;
}

interface AuthApiTokenPayload {
  userId: string;
}

// will figure out what we want to pass in here...
export function createAuthToken(userId: string) {
  const payload: AuthApiTokenPayload = {
    userId,
  };
  return createJWT(payload);
}

export function decodeAuthToken(token: string): AuthTokenData {
  const verified = verifyJWT(token);
  if (typeof verified === "string") {
    throw new Error(`Unexpected decoded token (should not be string): ${verified}`);
  }

  // Normalize the token (get userId, workspaceId and role)

  // V2 SDF token
  if ("version" in verified && verified.version === "2") {
    const { userId, workspaceId, role } = verified as SdfAuthTokenPayloadV2;
    return {
      userId, workspaceId, role, tokenId: verified.jti,
    };
  }

  // V1 SDF token
  if ("user_pk" in verified && "workspace_pk" in verified) {
    const { user_pk, workspace_pk } = verified as SdfAuthTokenPayloadV1;
    return { userId: user_pk, workspaceId: workspace_pk, role: "web" };
  }

  // Auth API token
  if ("userId" in verified) {
    const { userId } = verified as AuthApiTokenPayload;
    return { userId, workspaceId: undefined, role: "web" };
  }

  throw new Error(`Unsupported auth token format: ${JSON.stringify(verified)}`);
}

// Auth tokens used for communication between the user's browser and SDF
// and between that SDF instance and this auth api if necessary
export type SdfAuthTokenPayload = SdfAuthTokenPayloadV1 | SdfAuthTokenPayloadV2;
export const SdfAuthTokenRoles = ["web", "automation"] as const;
export type SdfAuthTokenRole = typeof SdfAuthTokenRoles[number];

interface SdfAuthTokenPayloadV2 {
  version: "2";
  userId: UserId;
  workspaceId: WorkspaceId;
  role: SdfAuthTokenRole;
}

// Old auth token versions
interface SdfAuthTokenPayloadV1 {
  version?: undefined;
  user_pk: UserId;
  workspace_pk: WorkspaceId;
}

// Pass a V2 token in.
export function createSdfAuthToken(
  payload: Omit<SdfAuthTokenPayloadV2, "version"> & { role: "automation" },
  options: (
    Omit<SignOptions, 'algorithm' | 'subject' | 'expiresIn' | 'jwtid'>
    & Required<Pick<SignOptions, 'expiresIn' | 'jwtid'>>
  ),
): string;
export function createSdfAuthToken(
  payload: Omit<SdfAuthTokenPayloadV2, "version"> & { role: "web" },
  options?: Omit<SignOptions, 'algorithm' | 'subject'>,
): string;
export function createSdfAuthToken(
  payload: Omit<SdfAuthTokenPayloadV2, "version">,
  options?: Omit<SignOptions, 'algorithm' | 'subject'>,
) {
  function createPayload(): SdfAuthTokenPayload {
    switch (payload.role) {
      case "web":
        // Generate V2 token if jwtid is provided (secure-bearer-tokens feature flag enabled)
        // Otherwise generate V1 token for backwards compatibility
        if (options?.jwtid) {
          return { version: "2", ...payload };
        }
        return { user_pk: payload.userId, workspace_pk: payload.workspaceId };
      case "automation":
        // Expire automation tokens quickly right now
        return { version: "2", ...payload };
      default:
        return payload.role satisfies never;
    }
  }

  return createJWT(createPayload(), { subject: payload.userId, ...(options ?? {}) });
}

export async function decodeSdfAuthToken(token: string) {
  return verifyJWT(token) as SdfAuthTokenPayload & JwtPayload;
}

export function normalizeSdfAuthTokenPayload(token: SdfAuthTokenPayload): Omit<SdfAuthTokenPayloadV2, "version"> {
  if ("user_pk" in token) {
    return {
      userId: token.user_pk,
      workspaceId: token.workspace_pk,
      role: "web",
    };
  } else {
    const { userId, workspaceId, role } = token;
    return { userId, workspaceId, role };
  }
}

function wipeAuthCookie(ctx: Koa.Context) {
  ctx.cookies.set(SI_COOKIE_NAME, null);
}

export const loadAuthMiddleware: Koa.Middleware<CustomAppState, CustomAppContext> = async (ctx, next) => {
  // Prioritize Authorization header over cookie (workspace tokens use headers)
  let authToken = ctx.headers.authorization?.split(" ").pop();
  if (!authToken) {
    authToken = ctx.cookies.get(SI_COOKIE_NAME);
  }
  if (!authToken) {
    // special auth handling only used in tests
    if (process.env.NODE_ENV === "test" && ctx.headers["spoof-auth"]) {
      ctx.state.token = { userId: ctx.headers["spoof-auth"] as string, role: "web" };
      const user = await getUserById(ctx.state.token.userId);
      if (!user) throw new Error("spoof auth user does not exist");
      ctx.state.authUser = user;
    }

    return next();
  }

  ctx.state.token = await tryCatch(() => {
    return decodeAuthToken(authToken!);
  }, (_err) => {
    // TODO: check the type of error before handling this way

    // clear the cookie and return an error
    wipeAuthCookie(ctx);
    throw new ApiError("Unauthorized", "AuthTokenCorrupt", "Invalid auth token");
  });

  // console.log(decoded);

  // make sure cookie is valid - not sure if this can happen...
  if (!ctx.state.token) {
    wipeAuthCookie(ctx);
    throw new ApiError("Unauthorized", "AuthTokenCorrupt", "Invalid auth token");
  }
  // TODO: deal with various other errors, logout on all devices, etc...

  const user = await getUserById(ctx.state.token.userId);

  if (!user) {
    wipeAuthCookie(ctx);
    throw new ApiError("Unauthorized", "AuthUserMissing", "Cannot find user data");
  }

  ctx.state.authUser = user;

  // Feature flag check - enforce tokenId validation if enabled
  const secureBearerToken = await posthog.isFeatureEnabled("secure-bearer-tokens", user.id);

  // Only enforce tokenId requirement for workspace-scoped tokens (SDF tokens)
  // Auth-portal session tokens (without workspaceId) don't need tokenId
  const isWorkspaceScopedToken = !!ctx.state.token.workspaceId;

  // If secure bearer tokens are enabled for this user, enforce V2 token requirements
  if (secureBearerToken && isWorkspaceScopedToken) {
    if (!ctx.state.token.tokenId) {
      wipeAuthCookie(ctx);
      throw new ApiError(
        "Unauthorized",
        "AuthTokenInvalid",
        "Token missing required identifier. Please log in again.",
      );
    }

    // Check if token is revoked in database
    const authToken = await getAuthToken(ctx.state.token.tokenId);
    if (!authToken || authToken.revokedAt) {
      wipeAuthCookie(ctx);
      throw new ApiError(
        "Unauthorized",
        "AuthTokenRevoked",
        "Auth token has been revoked",
      );
    }
    ctx.state.authToken = authToken;
  } else {
    // Legacy behavior: only check revocation if tokenId exists
    if (ctx.state.token.tokenId) {
      const authToken = await getAuthToken(ctx.state.token.tokenId);
      if (!authToken || authToken.revokedAt) {
        wipeAuthCookie(ctx);
        throw new ApiError("Unauthorized", "AuthTokenRevoked", "Auth token has been revoked");
      }
      ctx.state.authToken = authToken;
    }
  }

  // Make sure the workspace exists
  if (ctx.state.token.workspaceId) {
    const workspace = await getWorkspaceById(ctx.state.token.workspaceId);
    if (!workspace) {
      wipeAuthCookie(ctx);
      throw new ApiError("Unauthorized", "AuthWorkspaceMissing", "Cannot find workspace data");
    }
    ctx.state.authWorkspace = workspace;
  }

  return next();
};

export const requireWebTokenMiddleware: Koa.Middleware<CustomAppState, CustomAppContext> = async (ctx, next) => {
  if (ctx.state.token && ctx.state.token.role !== "web") {
    wipeAuthCookie(ctx);
    throw new ApiError("Unauthorized", "AutomationToken", "Automation tokens may not access the auth api");
  }

  return next();
};

export async function beginAuthConnect(workspace: Workspace, user: User) {
  // generate a new single use authentication code that we will send to the instance
  const connectCode = nanoid(24);
  await setCache(`auth:connect:${connectCode}`, {
    workspaceId: workspace.id,
    userId: user.id,
  }, { expiresIn: 60 });

  return await makeAuthConnectUrl(workspace, user, connectCode);
}

export async function makeAuthConnectUrl(workspace: Workspace, user: User, code: string, redirect?: string) {
  const params: { [key: string]: string } = { code, workspaceId: workspace.id };

  params.onDemandAssets = `true`;
  if (redirect) {
    params.redirect = redirect;
  }

  const paramsString = Object.keys(params).map((key) => `${key}=${params[key]}`).join("&");

  return `${workspace.instanceUrl}/auth-connect?${paramsString}`;
}
