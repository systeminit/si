import { globSync } from "glob";
import Router from "@koa/router";
import { createDeferredPromise } from "@si/ts-lib";
import { getThisDirname } from "../lib/this-file-path";
import {
  CustomAppContext,
  CustomAppState,
  CustomRouteContext,
} from "../custom-state";
import { ApiError } from "../lib/api-error";

const __dirname = getThisDirname(import.meta.url);

// Helpers
// There's a set of helpers both here and on specific route files that allow us
// to centralize error handling and make TS happier when dealing with params

/// Return auth user and fail if not present
export function extractAuthUser(ctx: CustomRouteContext) {
  const authUser = ctx.state.authUser;
  if (!authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  if (authUser.quarantinedAt !== null) {
    throw new ApiError("Unauthorized", `This account (ID ${authUser.id}) is quarantined. Contact SI support`);
  }

  return authUser;
}

/// Return auth user and fail if it's not an admin
export function extractAdminAuthUser(ctx: CustomRouteContext) {
  const authUser = extractAuthUser(ctx);

  if (!authUser.email.endsWith("@systeminit.com")) {
    throw new ApiError(
      "Forbidden",
      "You are not allowed to perform this operation",
    );
  }

  return authUser;
}

// we initialize and export the router immediately
// but we'll add routes to it here and in each routes file
export type CustomRouter = Router<CustomAppState, CustomAppContext>;
export const router = new Router<CustomAppState, CustomAppContext>();
export const automationApiRouter = new Router<CustomAppState, CustomAppContext>();

automationApiRouter.get("/", async (ctx) => {
  // TODO: add something which checks redis and postgres connections are working
  ctx.body = { systemStatus: "ok" };
});

// special route used to check 500 error handling is working correctly
if (process.env.NODE_ENV === "test") {
  router.get("/boom", async (ctx) => {
    // we'll look for this message in our tests to make sure it is not exposed
    throw new Error("unexpected error - crash boom bang");
  });
}

const routesLoadedDefer = createDeferredPromise();
export const routesLoaded = routesLoadedDefer.promise;

// automatically load all *.routes.ts files in this directory
// (need .js for when the files are built)
const routeFilePaths = globSync(`${__dirname}/**/*.routes.{js,ts}`);
// eslint-disable-next-line @typescript-eslint/no-floating-promises
(async function loadRoutes() {
  for (const routeFilePath of routeFilePaths) {
    // NOTE this is async, but in practice it's fine
    // if we see problems, we can switch over to importing manually...
    await import(routeFilePath.replace(__dirname, "./"));
  }
  routesLoadedDefer.resolve();
}());
