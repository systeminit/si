import _ from "lodash";
import { z } from "zod";
import { ApiError } from "../lib/api-error";
import { validate } from "../lib/validation-helpers";
import {
  findLatestTosForUser,
  saveTosAgreement,
} from "../services/tos.service";

import { CustomRouteContext } from "../custom-state";
import {
  getUserById,
  refreshUserAuth0Profile,
  saveUser,
} from "../services/users.service";
import { resendAuth0EmailVerification } from "../services/auth0.service";
import { router } from ".";

router.get("/whoami", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  ctx.body = {
    user: ctx.state.authUser,
  };
});

/// Return auth user id and fail if not present
function extractAuthUser(ctx: CustomRouteContext) {
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  if (ctx.state.authUser.quarantinedAt !== null) {
    throw new ApiError("Unauthorized", "This account is quarantined. Contact SI support");
  }

  return ctx.state.authUser;
}

async function extractUserIdParam(ctx: CustomRouteContext) {
  const userId = ctx.params.userId;
  if (!userId) {
    throw new Error("Only use this fn with routes containing :userId param");
  }

  const authUser = extractAuthUser(ctx);

  if (authUser.id === userId) {
    return authUser;
  } else {
    const user = await getUserById(userId);
    if (!user) {
      throw new ApiError("NotFound", "User not found");
    }

    return user;
  }
}

// :userId named param handler - little easier for TS this way than using router.param
async function extractOwnUserIdParam(ctx: CustomRouteContext) {
  if (!ctx.params.userId) {
    throw new Error("Only use this fn with routes containing :userId param");
  }

  // ensure user is logged in
  const authUser = extractAuthUser(ctx);

  // for now you can only edit yourself
  // eventually we may have SI admins able to edit everyone
  // or org admins able to edit people within their org...
  if (authUser.id !== ctx.params.userId) {
    throw new ApiError("Forbidden", "You can only edit your own info");
  }

  // we always have the user loaded already since you can only access yourself
  // but eventually we'd add a lookup by id and 404 handling
  return authUser;
}

router.patch("/users/:userId/quarantine", async (ctx) => {
  // Fail on bad auth user
  const authUser = extractAuthUser(ctx);

  // Fail on non SI user
  if (!authUser.email.endsWith("@systeminit.com")) {
    throw new ApiError(
      "Forbidden",
      "You are not allowed to perform this operation",
    );
  }

  const targetUser = await extractUserIdParam(ctx);

  if (targetUser.id === authUser.id) {
    throw new ApiError(
      "Forbidden",
      "An account cannot quarantine itself",
    );
  }

  const reqBody = validate(ctx.request.body, z.object({
    isQuarantined: z.boolean(),
  }));
  targetUser.quarantinedAt = reqBody.isQuarantined ? new Date() : null;

  await saveUser(targetUser);

  ctx.body = { user: targetUser };
});
router.patch("/users/:userId", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

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
  const user = await extractOwnUserIdParam(ctx);

  const reqBody = validate(ctx.request.body, z.object({
    step: z.string(),
  }));

  // using _.set fills in missing wrapper objects if necessary...
  _.set(user, ["onboardingDetails", "vroStepsCompletedAt", reqBody.step], new Date());

  await saveUser(user);

  ctx.body = { user };
});

router.post("/users/:userId/complete-profile", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  user.onboardingDetails ||= {};
  _.assign(user.onboardingDetails, ctx.request.body);

  if (!(user?.onboardingDetails as any)?.reviewedProfile) {
    _.set(user, ["onboardingDetails", "reviewedProfile"], new Date());
  }

  _.set(user, ["onboardingDetails", "firstTimeModal"], true);

  await saveUser(user);

  ctx.body = { user };
});

router.post("/users/:userId/refresh-auth0-profile", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);
  await refreshUserAuth0Profile(user);
  ctx.body = { user };
});
router.post("/users/:userId/resend-email-verification", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);
  if (!user.auth0Id) {
    throw new ApiError("Conflict", "User has no auth0 id");
  }
  if (user.emailVerified) {
    throw new ApiError("Conflict", "EmailAlreadyVerified", "Email is already verified");
  }
  await refreshUserAuth0Profile(user);
  if (user.emailVerified) {
    throw new ApiError("Conflict", "EmailAlreadyVerified", "Email is already verified");
  }

  await resendAuth0EmailVerification(user.auth0Id);
  ctx.body = { success: true };
});

router.get("/tos-details", async (ctx) => {
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }
  const latestTosVersion = await findLatestTosForUser(ctx.state.authUser);
  ctx.body = { tosVersion: latestTosVersion };
});

router.post("/tos-agreement", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  const reqBody = validate(ctx.request.body, z.object({
    // TODO: validate the version is a real one... need to decide on format and how it will be stored
    tosVersionId: z.string(),
  }));

  const userAgreedVersion = ctx.state.authUser.agreedTosVersion;
  if (userAgreedVersion && userAgreedVersion > reqBody.tosVersionId) {
    throw new ApiError("Conflict", "Cannot agree to earlier version of TOS");
  }
  const agreement = await saveTosAgreement(ctx.state.authUser, reqBody.tosVersionId, ctx.state.clientIp);
  ctx.body = agreement;
});

router.get("/users/:userId/firstTimeModal", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  ctx.body = { firstTimeModal: (user?.onboardingDetails as any)?.firstTimeModal };
});

router.post("/users/:userId/dismissFirstTimeModal", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  _.set(user, ["onboardingDetails", "firstTimeModal"], false);

  await saveUser(user);

  ctx.body = { firstTimeModal: (user?.onboardingDetails as any)?.firstTimeModal };
});
