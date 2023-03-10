import Router from "@koa/router";
import _ from 'lodash';
import { z } from 'zod';
import { ApiError } from "../lib/api-error";
import { validate } from "../lib/validation-helpers";
import { findLatestAgreedVersionForUser, findLatestTosForUser, saveTosAgreement } from "../services/tos.service";

import { router } from ".";

router.get("/whoami", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }

  ctx.body = {
    user: ctx.state.authUser,
  };
});

router.get("/tos-details", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', 'You are not logged in');
  }
  const latestTos = await findLatestTosForUser(ctx.state.authUser);
  ctx.body = _.omit(latestTos, 'markdown');
});

router.post("/tos-agreement", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }

  const reqBody = validate(ctx.request.body, z.object({
    // TODO: validate the version is a real one... need to decide on format and how it will be stored
    tosVersionId: z.string(),
  }));

  const latestTosVersion = await findLatestAgreedVersionForUser(ctx.state.authUser.id);
  if (latestTosVersion <= reqBody.tosVersionId) {
    throw new ApiError('Conflict', 'Cannot agree to earlier version of TOS');
  }
  const agreemenet = await saveTosAgreement(ctx.state.authUser.id, reqBody.tosVersionId, ctx.state.clientIp);
  ctx.body = agreemenet;
});
