import _ from "lodash";
import Router from "@koa/router";
import { ApiError } from "../lib/api-error";
import { getCache } from "../lib/cache";

export function initRoutes(router: Router) {
  router.get("/whoami", async (ctx) => {
    // user must be logged in
    if (!ctx.$.user) {
      throw new ApiError('Unauthorized', "You are not logged in");
    }

    ctx.body = {
      user: ctx.$.user,
    };
  });
}
