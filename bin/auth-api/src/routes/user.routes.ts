import _ from 'lodash';
import { z } from 'zod';
import { ApiError } from "../lib/api-error";
import { validate } from "../lib/validation-helpers";
import { findLatestTosForUser, saveTosAgreement } from "../services/tos.service";

import { CustomRouteContext } from '../custom-state';
import { saveUser } from '../services/users.service';
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

// :userId named param handler - little easier for TS this way than using router.param
async function handleUserIdParam(ctx: CustomRouteContext) {
  if (!ctx.params.userId) {
    throw new Error('Only use this fn with routes containing :userId param');
  }

  // ensure user is logged in
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', "You are not logged in");
  }

  // for now you can only edit yourself
  // eventually we may have SI admins able to edit everyone
  // or org admins able to edit people within their org...
  if (ctx.state.authUser.id !== ctx.params.userId) {
    throw new ApiError('Forbidden', "You can only edit your own info");
  }

  // we always have the user loaded already since you can only access yourself
  // but eventually we'd add a lookup by id and 404 handling
  return ctx.state.authUser;
}

router.patch("/users/:userId", async (ctx) => {
  const user = await handleUserIdParam(ctx);

  const reqBody = validate(ctx.request.body, z.object({
    // TODO: add checks on usernames looking right
    // TODO: figure out way to avoid marking everything as nullable
    firstName: z.string().nullable(),
    lastName: z.string().nullable(),
    nickname: z.string(),
    email: z.string().email(),
    pictureUrl: z.string().url().nullable(),
    discordUsername: z.string().nullable(),
    githubUsername: z.string().nullable(),
  }).partial());

  _.assign(user, reqBody);
  await saveUser(user);

  ctx.body = { user };
});

router.post("/users/:userId/complete-tutorial-step", async (ctx) => {
  const user = await handleUserIdParam(ctx);

  const reqBody = validate(ctx.request.body, z.object({
    step: z.string(),
  }));

  // using _.set fills in missing wrapper objects if necessary...
  _.set(user, ['onboardingDetails', 'vroStepsCompletedAt', reqBody.step], new Date());

  await saveUser(user);

  ctx.body = { user };
});

router.get("/tos-details", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError('Unauthorized', 'You are not logged in');
  }
  const latestTosVersion = await findLatestTosForUser(ctx.state.authUser);
  ctx.body = { tosVersion: latestTosVersion };
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

  const userAgreedVersion = ctx.state.authUser.agreedTosVersion;
  if (userAgreedVersion && userAgreedVersion > reqBody.tosVersionId) {
    throw new ApiError('Conflict', 'Cannot agree to earlier version of TOS');
  }
  const agreement = await saveTosAgreement(ctx.state.authUser, reqBody.tosVersionId, ctx.state.clientIp);
  ctx.body = agreement;
});
