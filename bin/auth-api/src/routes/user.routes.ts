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
  getQuarantinedUsers,
  getSuspendedUsers,
  getUserById,
  getUserSignupReport,
  refreshUserAuth0Profile,
  saveUser,
} from "../services/users.service";
import { resendAuth0EmailVerification } from "../services/auth0.service";
import { extractAdminAuthUser, extractAuthUser, router } from ".";

router.get("/whoami", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  ctx.body = {
    user: ctx.state.authUser,
  };
});

// :userId named param handler - little easier for TS this way than using router.param
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

// Extract user based on :userId param, fail if not equal to auth user
async function extractOwnUserIdParam(ctx: CustomRouteContext) {
  if (!ctx.params.userId) {
    throw new Error("Only use this fn with routes containing :userId param");
  }

  // ensure user is logged in
  const authUser = extractAuthUser(ctx);

  if (authUser.id !== ctx.params.userId) {
    throw new ApiError("Forbidden", "You can only edit your own info");
  }

  // we always have the auth user loaded already
  return authUser;
}

router.patch("/users/:userId/quarantine", async (ctx) => {
  // Fail on bad auth user
  const authUser = extractAdminAuthUser(ctx);

  const targetUser = await extractUserIdParam(ctx);

  if (targetUser.id === authUser.id) {
    throw new ApiError("Forbidden", "An account cannot quarantine itself");
  }

  const reqBody = validate(
    ctx.request.body,
    z.object({
      isQuarantined: z.boolean(),
    }),
  );
  targetUser.quarantinedAt = reqBody.isQuarantined ? new Date() : null;

  await saveUser(targetUser);

  ctx.body = { user: targetUser };
});

router.patch("/users/:userId/suspend", async (ctx) => {
  // Fail on bad auth user
  const authUser = extractAdminAuthUser(ctx);

  const targetUser = await extractUserIdParam(ctx);

  if (targetUser.id === authUser.id) {
    throw new ApiError("Forbidden", "An account cannot suspend itself");
  }

  const reqBody = validate(
    ctx.request.body,
    z.object({
      isSuspended: z.boolean(),
    }),
  );
  targetUser.suspendedAt = reqBody.isSuspended ? new Date() : null;

  await saveUser(targetUser);

  ctx.body = { user: targetUser };
});

export type SuspendedUser = {
  userId: string;
  email: string;
  suspendedAt: Date | null;
};
router.get("/users/suspended", async (ctx) => {
  extractAdminAuthUser(ctx);

  const suspended: SuspendedUser[] = [];
  const suspendedUsers = await getSuspendedUsers();

  suspendedUsers.forEach((sm) => {
    suspended.push({
      userId: sm.id,
      email: sm.email,
      suspendedAt: sm.suspendedAt,
    });
  });

  ctx.body = suspended;
});

export type QuarantinedUser = {
  userId: string;
  email: string;
  quarantinedAt: Date | null;
};
router.get("/users/quarantined", async (ctx) => {
  extractAdminAuthUser(ctx);

  const quarantined: QuarantinedUser[] = [];
  const quarantinedUsers = await getQuarantinedUsers();

  quarantinedUsers.forEach((qm) => {
    quarantined.push({
      userId: qm.id,
      email: qm.email,
      quarantinedAt: qm.quarantinedAt,
    });
  });

  ctx.body = quarantined;
});

export type SignupUsersReport = {
  firstName?: string | null;
  lastName?: string | null;
  email: string;
  signupMethod: string;
  discordUsername?: string | null;
  githubUsername?: string | null;
  signupAt: Date | null;
};
router.get("/users/report", async (ctx) => {
  extractAdminAuthUser(ctx);

  const reqBody = validate(
    ctx.request.query,
    z.object({
      startDate: z.string(),
      endDate: z.string(),
    }),
  );

  const reportUsers: SignupUsersReport[] = [];
  const signups = await getUserSignupReport(
    new Date(reqBody.startDate),
    new Date(reqBody.endDate),
  );

  signups.forEach((u) => {
    reportUsers.push({
      firstName: u.firstName,
      lastName: u.lastName,
      email: u.email,
      discordUsername: u.discordUsername,
      githubUsername: u.githubUsername,
      signupAt: u.signupAt,
      signupMethod: extractAuthProvider(u.auth0Id),
    });
  });

  ctx.body = reportUsers;
});

function extractAuthProvider(authId: string | null): string {
  if (!authId) return "unknown";

  const parts = authId.split("|");
  return parts[0] || authId;
}

router.patch("/users/:userId", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z
      .object({
        // TODO: add checks on usernames looking right
        // TODO: figure out way to avoid marking everything as nullable
        firstName: z.string().nullable(),
        lastName: z.string().nullable(),
        nickname: z.string(),
        email: z.string().email(),
        pictureUrl: z.string().url().nullable(),
        discordUsername: z.string().nullable(),
        githubUsername: z.string().nullable(),
      })
      .partial(),
  );

  _.assign(user, reqBody);
  await saveUser(user);

  ctx.body = { user };
});

router.post("/users/:userId/complete-tutorial-step", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z.object({
      step: z.string(),
    }),
  );

  // using _.set fills in missing wrapper objects if necessary...
  _.set(
    user,
    ["onboardingDetails", "vroStepsCompletedAt", reqBody.step],
    new Date(),
  );

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
    throw new ApiError(
      "Conflict",
      "EmailAlreadyVerified",
      "Email is already verified",
    );
  }
  await refreshUserAuth0Profile(user);
  if (user.emailVerified) {
    throw new ApiError(
      "Conflict",
      "EmailAlreadyVerified",
      "Email is already verified",
    );
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

  const reqBody = validate(
    ctx.request.body,
    z.object({
      // TODO: validate the version is a real one... need to decide on format and how it will be stored
      tosVersionId: z.string(),
    }),
  );

  const userAgreedVersion = ctx.state.authUser.agreedTosVersion;
  if (userAgreedVersion && userAgreedVersion > reqBody.tosVersionId) {
    throw new ApiError("Conflict", "Cannot agree to earlier version of TOS");
  }
  const agreement = await saveTosAgreement(
    ctx.state.authUser,
    reqBody.tosVersionId,
    ctx.state.clientIp,
  );
  ctx.body = agreement;
});

router.get("/users/:userId/firstTimeModal", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  ctx.body = {
    firstTimeModal: (user?.onboardingDetails as any)?.firstTimeModal,
  };
});

router.post("/users/:userId/dismissFirstTimeModal", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  _.set(user, ["onboardingDetails", "firstTimeModal"], false);

  await saveUser(user);

  ctx.body = {
    firstTimeModal: (user?.onboardingDetails as any)?.firstTimeModal,
  };
});
