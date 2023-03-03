import _ from "lodash";
import Router from "@koa/router";
import { ApiError } from "../lib/api-error";
import { createJWT } from "../lib/jwt";
import authService from "../services/auth-service";

export function initRoutes(router: Router) {
  router.get("/auth/login", async (ctx) => {
    // TODO: can read from querystring info about where request originated
    // so that we can shoot later directly back to the right place, skipping auth portal

    const { url } = authService.getAuth0LoginUrl();
    ctx.redirect(url);
  });

  router.get("/auth/auth-callback", async (ctx) => {
    // const { code, state } = ctx.request.query;
    // TODO: find a better way to assert/check its not a string array (and make TS happy)
    // (validation tooling should do this)
    const code = ctx.request.query.code as string;
    const state = ctx.request.query.state as string;

    // TODO: swap in validation tooling
    if (!code) throw new ApiError("BadRequest", "Missing required param - code");
    if (!state) throw new ApiError("BadRequest", "Missing required param - state");
    // TODO: check state/code look like valid values

    const { profile, token } = await authService.completeAuth0TokenExchange(code);

    // TODO: create/update user, send to posthog, etc...

    const siToken = createJWT({
      userId: profile.sub, // this is auth0's user id, swap for our new id
      email: profile.email,
    });

    ctx.cookies.set("si-auth", siToken, {
      httpOnly: true,
      secure: true, // TODO: toggle this on if not running locally
      // domain:,
    });

    // ctx.body = { authToken: siToken, profile };

    ctx.redirect(`${process.env.AUTH_PORTAL_URL}/dashboard`);
  });

  router.get("/auth/logout", async (ctx) => {
    ctx.redirect(authService.getAuth0LogoutUrl());
  });

  router.get("/auth/logout-callback", async (ctx) => {
    console.log("Logged out!");
    ctx.body = { logout: true };
  });
}
