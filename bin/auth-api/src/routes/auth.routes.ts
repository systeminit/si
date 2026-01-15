import { z } from "zod";
import { nanoid } from "nanoid";
import { ulid } from "ulidx";
import { ApiError } from "../lib/api-error";
import {
  completeAuth0TokenExchange,
  getAuth0LoginUrl,
  getAuth0LogoutUrl,
  getAuth0UserCredential,
} from "../services/auth0.service";
import {
  createAuthToken,
  createSdfAuthToken,
  decodeSdfAuthToken,
  SI_COOKIE_NAME,
} from "../services/auth.service";
import {
  registerAuthToken,
  updateAuthToken,
} from "../services/auth_tokens.service";
import { getCache, setCache } from "../lib/cache";
import {
  createOrUpdateUserFromAuth0Details,
  getUserByEmail,
  getUserById,
} from "../services/users.service";
import { validate } from "../lib/validation-helpers";
import { posthog } from "../lib/posthog";
import { tracker } from "../lib/tracker";

import { userRoleForWorkspace } from "../services/workspaces.service";
import { router } from ".";

const FALLBACK_REDIR_PORT = 9003;

const parseCliRedirectPort = (port: string[] | string): number => {
  if (typeof port !== "string") {
    return FALLBACK_REDIR_PORT;
  }

  const portNumber = Number.parseInt(port);
  if (isNaN(portNumber) || portNumber < 1) {
    return FALLBACK_REDIR_PORT;
  }

  return portNumber;
};

router.get("/auth/login", async (ctx) => {
  // passing in cli_redir=PORT_NO will begin the auth flow for the si cli
  const cliRedirParam = ctx.request.query.cli_redir;
  const cliRedirect = cliRedirParam
    ? parseCliRedirectPort(cliRedirParam)
    : undefined;

  // passing in a querystring signup=1 will show auth0's signup page instead of login
  // it's almost exactly the same, but one less step if using a password
  const { randomState, url } = getAuth0LoginUrl(!!ctx.request.query.signup);
  // save our auth request in the cache using our random state
  await setCache(
    `auth:start:${randomState}`,
    {
      cliRedirect,
    },
    { expiresIn: 300 }, // expire in 5 minutes
  );

  // redirects to Auth0 to actually log in
  ctx.redirect(url);
});

router.post("/auth/login", async (ctx) => {
  const { email, password, workspaceId } = validate(
    ctx.request.body,
    z.object({
      email: z.string(),
      password: z.string(),
      workspaceId: z.string(),
    }),
  );

  const user = await getUserByEmail(email);
  if (!user) {
    throw new ApiError("Forbidden", "Bad user");
  }

  try {
    await getAuth0UserCredential(email, password);
  } catch (e) {
    let message = "Bad User";
    if (e instanceof Error) {
      message = e.message;
    }
    throw new ApiError("Forbidden", message);
  }

  const memberRole = await userRoleForWorkspace(user.id, workspaceId);
  if (!memberRole) {
    throw new ApiError("Forbidden", "You do not have access to that workspace");
  }

  // Feature flag check - generate V2 token with tracking if enabled
  const secureBearerToken = await posthog.isFeatureEnabled(
    "secure-bearer-tokens",
    user.id,
  );

  let token: string;
  if (secureBearerToken) {
    // Generate V2 token with jti and expiration
    const jti = ulid();
    token = createSdfAuthToken(
      {
        userId: user.id,
        workspaceId,
        role: "web",
      },
      { jwtid: jti, expiresIn: "30d" },
    );

    // Register token in database
    const decoded = await decodeSdfAuthToken(token);
    await registerAuthToken("Web Session", decoded);
  } else {
    // Legacy V1 token
    token = createSdfAuthToken({
      userId: user.id,
      workspaceId,
      role: "web",
    });
  }

  ctx.body = { token };
});

router.get("/auth/login-callback", async (ctx) => {
  const reqQuery = validate(
    ctx.request.query,
    z.object({
      code: z.string(),
      state: z.string(),
    }),
  );

  // verify `state` matches ours by checking cache (and destroys key so it cannot be used twice)
  const authStartMeta = await getCache(`auth:start:${reqQuery.state}`, true);
  if (!authStartMeta) {
    throw new ApiError("Conflict", "Oauth state does not match");
  }

  const { profile } = await completeAuth0TokenExchange(reqQuery.code);
  const user = await createOrUpdateUserFromAuth0Details(profile);

  // create new JWT used when communicating between the user's browser and
  // _this_ API (via secure http cookie)
  const siToken = createAuthToken(user.id);

  ctx.cookies.set(SI_COOKIE_NAME, siToken, {
    httpOnly: true,
    secure: (process.env.AUTH_API_URL as string).startsWith("https://"),
  });

  const cliRedir = authStartMeta.cliRedirect;
  if (cliRedir) {
    const nonce = nanoid(32);
    await setCache(
      `auth:cli:${nonce}`,
      {
        token: siToken,
      },
      { expiresIn: 500 },
    );
    ctx.redirect(`http://localhost:${cliRedir}?nonce=${nonce}`);
  } else {
    ctx.redirect(`${process.env.AUTH_PORTAL_URL}/login-success`);
  }
});

router.get("/auth/cli-auth-api-token", async (ctx) => {
  const nonce = ctx.request.query.nonce;
  if (!nonce) {
    throw new ApiError("BadRequest", "Nonce required");
  }

  const apiToken = await getCache(`auth:cli:${nonce}`, true);
  if (!apiToken) {
    throw new ApiError("Conflict", "Invalid or expired nonce");
  }

  ctx.body = apiToken;
});

router.post("/session/logout", async (ctx) => {
  let authToken = ctx.headers.authorization?.split(" ").pop();
  if (!authToken) {
    authToken = ctx.cookies.get(SI_COOKIE_NAME);
  }

  if (!authToken) {
    throw new ApiError("Unauthorized", "No authentication token provided");
  }

  try {
    const decoded = await decodeSdfAuthToken(authToken);
    const user = await getUserById(decoded.userId);

    if (!user) {
      throw new ApiError("Unauthorized", "Invalid authentication token");
    }

    // Check feature flag for this user
    const secureBearerToken = await posthog.isFeatureEnabled(
      "secure-bearer-tokens",
      decoded.userId,
    );

    // If secure bearer tokens are enabled and token has jti, revoke it
    if (secureBearerToken && decoded.jti) {
      await updateAuthToken(decoded.jti, { revokedAt: new Date() });

      // Track token revocation event
      tracker.trackEvent(user, "auth_token_revoked", {
        tokenId: decoded.jti,
        workspaceId: decoded.workspaceId,
        revokedAt: new Date(),
        revokedBy: user.email,
        revocationMethod: "user_logout",
        tokenFormat: "v2",
      });
    }
  } catch (error) {
    throw new ApiError("Unauthorized", "Invalid authentication token");
  }

  ctx.body = { success: true };
});

router.get("/auth/logout", async (ctx) => {
  // we wont check if user is logged in because even without an auth cookie from us
  // they could still be logged in on auth0, and forwarding to auth0 logout
  // will log them out there as well

  // Try to revoke the token if it exists
  try {
    let authToken = ctx.cookies.get(SI_COOKIE_NAME);
    if (!authToken && ctx.headers.authorization) {
      authToken = ctx.headers.authorization.split(" ").pop();
    }

    if (authToken) {
      const decoded = await decodeSdfAuthToken(authToken);
      const user = await getUserById(decoded.userId);

      // Check feature flag for this user
      const secureBearerToken = await posthog.isFeatureEnabled(
        "secure-bearer-tokens",
        decoded.userId,
      );

      // If secure bearer tokens are enabled and token has jti, revoke it
      if (secureBearerToken && decoded.jti && user) {
        await updateAuthToken(decoded.jti, { revokedAt: new Date() });

        // Track token revocation event
        tracker.trackEvent(user, "auth_token_revoked", {
          tokenId: decoded.jti,
          workspaceId: decoded.workspaceId,
          revokedAt: new Date(),
          revokedBy: user.email,
          revocationMethod: "auth_portal_logout",
          tokenFormat: "v2",
        });
      }
    }
  } catch (error) {
    // Don't fail logout if token revocation fails
  }

  // clear our auth cookie
  ctx.cookies.set(SI_COOKIE_NAME, null);
  // forward to auth0 which will log them out on auth0
  ctx.redirect(getAuth0LogoutUrl());
});

router.get("/auth/logout-callback", async (ctx) => {
  // console.log("Logged out!");
  // ctx.body = { logout: true };
  ctx.redirect(`${process.env.AUTH_PORTAL_URL}/logout-success`);
});
