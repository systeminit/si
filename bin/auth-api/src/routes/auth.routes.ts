import _ from "lodash";
import Router from "@koa/router";
import { ApiError } from "../lib/api-error";
import { createJWT } from "../lib/jwt";
import {
  completeAuth0TokenExchange, createAuthToken, getAuth0LoginUrl, getAuth0LogoutUrl, SI_COOKIE_NAME,
} from "../services/auth";
import { setCache, getCache } from "../lib/cache";
import {
  createUser, getUserById, updateUser, User,
} from "../services/users.service";

export function initRoutes(router: Router) {
  router.get("/auth/login", async (ctx) => {
    // TODO: can read from querystring info about where request originated
    // so that we can shoot later directly back to the right place, skipping auth portal

    const { randomState, url } = getAuth0LoginUrl();

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

  router.get("/auth/login-callback", async (ctx) => {
    // const { code, state } = ctx.request.query;
    // TODO: find a better way to assert/check its not a string array (and make TS happy)
    // (validation tooling should do this)
    const code = ctx.request.query.code as string;
    const state = ctx.request.query.state as string;

    // TODO: swap in validation tooling
    if (!code) throw new ApiError("BadRequest", "Missing required param - code");
    if (!state) throw new ApiError("BadRequest", "Missing required param - state");
    // TODO: check state/code look like valid values

    // verify `state` matches ours by checking cache (and destroys key so it cannot be used twice)
    const authStartMeta = await getCache(`auth:start:${state}`, true);
    if (!authStartMeta) {
      throw new ApiError('Conflict', 'Oauth state does not match');
    }

    const { profile, token } = await completeAuth0TokenExchange(code);
    const userId = profile.sub;

    // TEMPORARY - shove user data in cache while we dont have a db
    const userFromAuth0Profile: User = {
      id: profile.sub,
      firstName: profile.given_name,
      lastName: profile.family_name,
      pictureUrl: profile.picture,
      email: profile.email,
      emailVerified: profile.emailVerified,
      // auth0Details: profile,
    };
    const existingUser = await getUserById(userId);
    if (existingUser) {
      await updateUser(userFromAuth0Profile);
    } else {
      await createUser(userFromAuth0Profile);
    }

    // TODO: create/update user, send to posthog, etc...

    // create new JWT used when communicating between the user's browser and _this_ API (via secure http cookie)
    const siToken = createAuthToken(userId);

    ctx.cookies.set(SI_COOKIE_NAME, siToken, {
      // TODO: verify these settings
      httpOnly: true,
      // secure: true, // IMPORTANT - turn this on if domain is not localhost
      // domain:,
    });

    // ctx.body = { authToken: siToken, profile };

    ctx.redirect(`${process.env.AUTH_PORTAL_URL}/dashboard`);
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
    ctx.redirect(`${process.env.AUTH_PORTAL_URL}/login`);
  });
}
