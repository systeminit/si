import { z } from "zod";
import { ApiError } from "../lib/api-error";
import {
  completeAuth0TokenExchange,
  getAuth0LoginUrl,
  getAuth0LogoutUrl,
  getAuth0UserCredential,
} from "../services/auth0.service";
import {
  SI_COOKIE_NAME,
  createAuthToken,
  createSdfAuthToken,
} from "../services/auth.service";
import { setCache, getCache } from "../lib/cache";
import {
  createOrUpdateUserFromAuth0Details, getUserByEmail,
} from "../services/users.service";
import { validate } from "../lib/validation-helpers";

import {
  userRoleForWorkspace,
} from "../services/workspaces.service";
import { router } from ".";

router.get("/auth/login", async (ctx) => {
  // TODO: can read from querystring info about where request originated
  // so that we can shoot later directly back to the right place, skipping auth portal

  // passing in a querystring signup=1 will show auth0's signup page instead of login
  // it's almost exactly the same, but one less step if using a password
  const { randomState, url } = getAuth0LoginUrl(!!ctx.request.query.signup);

  // save our auth request in the cache using our random state
  await setCache(
    `auth:start:${randomState}`,
    {
      // here we'll save info about the request
      // like extra query params about where they came from...
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

  const token = createSdfAuthToken({
    userId: user.id,
    workspaceId,
    role: "web",
  });

  ctx.body = { token };
});

router.get("/auth/login-callback", async (ctx) => {
  const reqQuery = validate(ctx.request.query, z.object({
    // TODO: could check state/code look like valid values
    code: z.string(),
    state: z.string(),
  }));

  // verify `state` matches ours by checking cache (and destroys key so it cannot be used twice)
  const authStartMeta = await getCache(`auth:start:${reqQuery.state}`, true);
  if (!authStartMeta) {
    throw new ApiError("Conflict", "Oauth state does not match");
  }

  const { profile } = await completeAuth0TokenExchange(reqQuery.code);
  const user = await createOrUpdateUserFromAuth0Details(profile);
  // TODO: create/update user, send to posthog, etc...

  // create new JWT used when communicating between the user's browser and _this_ API (via secure http cookie)
  const siToken = createAuthToken(user.id);

  ctx.cookies.set(SI_COOKIE_NAME, siToken, {
    // TODO: verify these settings
    httpOnly: true,
    secure: (process.env.AUTH_API_URL as string).startsWith("https://"),
    // domain:,
  });

  // ctx.body = { authToken: siToken, profile };

  ctx.redirect(`${process.env.AUTH_PORTAL_URL}/login-success`);
});

router.get("/auth/logout", async (ctx) => {
  // we wont check if user is logged in because even without an auth cookie from us
  // they could still be logged in on auth0, and forwarding to auth0 logout
  // will log them out there as well

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
