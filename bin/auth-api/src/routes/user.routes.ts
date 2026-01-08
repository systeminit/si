import _ from "lodash";
import { z } from "zod";
import { TosVersion } from "@si/ts-lib/src/terms-of-service";
import { ApiError } from "../lib/api-error";
import {
  ALLOWED_INPUT_REGEX,
  DISCORD_TAG_REGEX,
  GITHUB_USERNAME_REGEX,
  NAME_REGEX,
  NICKNAME_REGEX,
  URL_DETECTION_REGEX,
  validate,
  MAX_LENGTH_STANDARD,
} from "../lib/validation-helpers";
import {
  findLatestTosForUser,
  saveTosAgreement,
} from "../services/tos.service";

import { CustomRouteContext } from "../custom-state";
import {
  create_lago_customer_records,
  getQuarantinedUsers,
  getSuspendedUsers,
  getUserById,
  getUsersByEmail,
  getUserSignupReport,
  refreshUserAuth0Profile,
  saveUser,
} from "../services/users.service";
import { resendAuth0EmailVerification } from "../services/auth0.service";
import { tracker } from "../lib/tracker";
import { createProductionWorkspaceForUser } from "../services/workspaces.service";
import {
  CustomerDetail,
  generateCustomerCheckoutUrl,
  getCustomerActiveSubscription,
  getCustomerBillingDetails,
  getCustomerPortalUrl,
  updateCustomerDetails,
} from "../lib/lago";
import { checkCustomerPaymentMethodSet } from "../lib/stripe";
import {
  automationApiRouter, extractAdminAuthUser, extractAuthUser, router,
} from ".";

automationApiRouter.get("/whoami", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  ctx.body = {
    user: ctx.state.authUser,
    authToken: ctx.state.authToken,
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

  const quarantineDate = new Date();
  if (reqBody.isQuarantined) {
    tracker.trackEvent(targetUser, "quarantine_user", {
      quarantinedBy: authUser.email,
      quarantinedAt: quarantineDate,
    });
  } else {
    tracker.trackEvent(targetUser, "unquarantine_user", {
      unQuarantinedBy: authUser.email,
      unQuarantinedAt: quarantineDate,
    });
  }

  targetUser.quarantinedAt = reqBody.isQuarantined ? quarantineDate : null;

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

  const suspensionDate = new Date();
  if (reqBody.isSuspended) {
    tracker.trackEvent(targetUser, "suspend_user", {
      suspendedBy: authUser.email,
      suspendedAt: suspensionDate,
    });
  } else {
    tracker.trackEvent(targetUser, "unsuspend_user", {
      unSuspendedBy: authUser.email,
      unSuspendedAt: suspensionDate,
    });
  }

  targetUser.suspendedAt = reqBody.isSuspended ? suspensionDate : null;

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

router.get("/users/by-email", async (ctx) => {
  extractAdminAuthUser(ctx);

  const reqBody = validate(
    ctx.request.query,
    z.object({
      email: z.string(),
    }),
  );

  const users = await getUsersByEmail(reqBody.email);

  ctx.body = users;
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
        // Name fields use NAME_REGEX to prevent URLs and domain names
        firstName: z.string()
          .max(MAX_LENGTH_STANDARD, `First name must be ${MAX_LENGTH_STANDARD} characters or less`)
          .regex(NAME_REGEX, "First name contains invalid characters or URLs")
          .refine((val) => !URL_DETECTION_REGEX.test(val), {
            message: "URLs are not allowed in first name",
          })
          .nullable(),
        lastName: z.string()
          .max(MAX_LENGTH_STANDARD, `Last name must be ${MAX_LENGTH_STANDARD} characters or less`)
          .regex(NAME_REGEX, "Last name contains invalid characters or URLs")
          .refine((val) => !URL_DETECTION_REGEX.test(val), {
            message: "URLs are not allowed in last name",
          })
          .nullable(),
        nickname: z.string()
          .max(MAX_LENGTH_STANDARD, `Nickname must be ${MAX_LENGTH_STANDARD} characters or less`)
          .regex(NICKNAME_REGEX, "Nickname contains invalid characters or URLs")
          .refine((val) => !URL_DETECTION_REGEX.test(val), {
            message: "URLs are not allowed in nickname",
          }),
        email: z.string().email(),
        pictureUrl: z.string()
          .url()
          .refine((url) => {
            try {
              const parsed = new URL(url);
              return ['http:', 'https:'].includes(parsed.protocol);
            } catch {
              return false;
            }
          }, { message: "Only HTTP/HTTPS URLs are allowed for profile pictures" })
          .nullable(),
        // Discord username - supports both new format and old discriminator format
        discordUsername: z.union([
          z.string()
            .max(MAX_LENGTH_STANDARD, `Discord username must be ${MAX_LENGTH_STANDARD} characters or less`)
            .regex(DISCORD_TAG_REGEX, "Invalid Discord username format"),
          z.literal(""),
          z.null(),
        ]).transform((val) => (val === "" ? null : val)),
        // GitHub username - follows official GitHub username rules
        githubUsername: z.union([
          z.string()
            .max(MAX_LENGTH_STANDARD, `GitHub username must be ${MAX_LENGTH_STANDARD} characters or less`)
            .regex(GITHUB_USERNAME_REGEX, "Invalid GitHub username format"),
          z.literal(""),
          z.null(),
        ]).transform((val) => (val === "" ? null : val)),
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

  const tosVersion = ctx.state.authUser.agreedTosVersion === latestTosVersion ? null : latestTosVersion;
  ctx.body = { tosVersion };
});

router.post("/tos-agreement", async (ctx) => {
  // user must be logged in
  if (!ctx.state.authUser) {
    throw new ApiError("Unauthorized", "You are not logged in");
  }

  // Extract values of enum to array, and type cast it to the type zod needs to creat its enum validation
  // the type casted to below just means the function expects at least one entry in the array,
  // which we know we provide.
  const tosVersionIds = Object.values(TosVersion) as [string, ...string[]];

  const reqBody = validate(
    ctx.request.body,
    z.object({
      tosVersionId: z.enum(tosVersionIds),
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

  if (userAgreedVersion) {
    // This means we have a user that has accepted an old ToS and is prompted for the latest ToS!
    // We need to create them a production workspace if they don't already have one during the Cohort
    await createProductionWorkspaceForUser(ctx.state.authUser.id);

    // Map that a user has upgraded to the new ToS and opted in to being a customer
    tracker.trackEvent(
      ctx.state.authUser,
      "existing_user_subscription_create",
      {
        signedUpAt: new Date(),
      },
    );
    // Create the lago account and the billing user
    await create_lago_customer_records(ctx.state.authUser);
  }

  ctx.body = agreement;
});

router.post("/users/:userId/dismissFirstTimeModal", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  _.set(user, ["onboardingDetails", "firstTimeModal"], false);

  await saveUser(user);

  ctx.body = {
    firstTimeModal: (user?.onboardingDetails as any)?.firstTimeModal,
  };
});

router.get("/users/:userId/firstTimeModal", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  ctx.body = {
    firstTimeModal: (user?.onboardingDetails as any)?.firstTimeModal,
  };
});

router.post("/users/:userId/create-billing-integration", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  await create_lago_customer_records(user);

  ctx.body = { success: true };
});

router.patch("/users/:userId/billingDetails", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  const reqBody = validate(
    ctx.request.body,
    z
      .object({
        // Billing names use NAME_REGEX to prevent URLs
        firstName: z.string()
          .max(MAX_LENGTH_STANDARD, `First name must be ${MAX_LENGTH_STANDARD} characters or less`)
          .regex(NAME_REGEX, "First name contains invalid characters")
          .refine((val) => !URL_DETECTION_REGEX.test(val), {
            message: "URLs are not allowed in first name",
          }),
        lastName: z.string()
          .max(MAX_LENGTH_STANDARD, `Last name must be ${MAX_LENGTH_STANDARD} characters or less`)
          .regex(NAME_REGEX, "Last name contains invalid characters")
          .refine((val) => !URL_DETECTION_REGEX.test(val), {
            message: "URLs are not allowed in last name",
          }),
        companyInformation: z.object({
          // Company information uses ALLOWED_INPUT_REGEX (allows some punctuation)
          legalName: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          legalNumber: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          taxIdentificationNumber: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          phoneNumber: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
        }),
        billingInformation: z.object({
          // Address fields use ALLOWED_INPUT_REGEX
          addressLine1: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          addressLine2: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          zipCode: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          city: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          state: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
          country: z.string().regex(ALLOWED_INPUT_REGEX).nullable(),
        }),
      })
      .partial(),
  );

  // Let's update any changes to first name or last name that the user has specified!
  if (
    user.firstName !== reqBody.firstName || user.lastName !== reqBody.lastName
  ) {
    _.assign(user.firstName, reqBody.firstName);
    _.assign(user.lastName, reqBody.lastName);
    await saveUser(user);
  }

  const customer: CustomerDetail = {
    id: user.id,
    email: user.email,
    firstName: reqBody.firstName,
    lastName: reqBody.lastName,
    customerCheckoutUrl: "",
    customerPortalUrl: "",
    companyInformation: {},
    billingInformation: {},
  };

  if (reqBody.billingInformation) {
    customer.billingInformation.addressLine1 = reqBody.billingInformation.addressLine1;
    customer.billingInformation.addressLine2 = reqBody.billingInformation.addressLine2;
    customer.billingInformation.zipCode = reqBody.billingInformation.zipCode;
    customer.billingInformation.city = reqBody.billingInformation.city;
    customer.billingInformation.state = reqBody.billingInformation.state;
    customer.billingInformation.country = reqBody.billingInformation.country;
  }

  if (reqBody.companyInformation) {
    customer.companyInformation.legalName = reqBody.companyInformation.legalName;
    customer.companyInformation.legalNumber = reqBody.companyInformation.legalNumber;
    customer.companyInformation.taxIdentificationNumber = reqBody.companyInformation.taxIdentificationNumber;
    customer.companyInformation.phoneNumber = reqBody.companyInformation.phoneNumber;
  }

  await updateCustomerDetails(customer);

  ctx.body = { success: true };
});

router.get("/users/:userId/billingDetails", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);
  const lagoCustomer = await getCustomerBillingDetails(user.id);
  if (!lagoCustomer) {
    throw new ApiError(
      "InternalServerError",
      "Unable to find the customer details",
    );
  }

  const customerCheckoutUrl = await generateCustomerCheckoutUrl(user.id);
  if (!customerCheckoutUrl) {
    throw new ApiError(
      "InternalServerError",
      "Unable to generate customer checkout url",
    );
  }

  const customerPortalUrl = await getCustomerPortalUrl(user.id);
  if (!customerPortalUrl) {
    throw new ApiError(
      "InternalServerError",
      "Unable to get customer portal url",
    );
  }

  // Should we check that the email in Lago is the same as our database?
  // do we care??
  const billingDetails = {
    id: user.id,
    firstName: user.firstName,
    lastName: user.lastName,
    email: lagoCustomer.email,
    companyInformation: {
      legalName: lagoCustomer.legal_name,
      legalNumber: lagoCustomer.legal_number,
      taxIdentificationNumber: lagoCustomer.tax_identification_number,
      phoneNumber: lagoCustomer.phone,
    },
    billingInformation: {
      addressLine1: lagoCustomer.address_line1,
      addressLine2: lagoCustomer.address_line2,
      zipCode: lagoCustomer.zipcode,
      city: lagoCustomer.city,
      state: lagoCustomer.state,
      country: lagoCustomer.country,
    },
    customerCheckoutUrl,
    customerPortalUrl,
  };

  ctx.body = { billingDetails };
});

router.get("/users/:userId/activeSubscription", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  const activeUser = await getCustomerBillingDetails(user.id);
  if (!activeUser) {
    ctx.body = {};
  }

  const activeSubscription = await getCustomerActiveSubscription(user.id);
  ctx.body = { activeSubscription };
});

router.get("/users/:userId/hasBillingDetails", async (ctx) => {
  const user = await extractOwnUserIdParam(ctx);

  const activeUser = await getCustomerBillingDetails(user.id);
  if (!activeUser) {
    ctx.body = {};
  }

  let paymentDetailsSet = false;
  if (activeUser?.billing_configuration?.provider_customer_id) {
    paymentDetailsSet = await checkCustomerPaymentMethodSet(
      activeUser?.billing_configuration?.provider_customer_id,
    );
  }

  ctx.body = { paymentDetailsSet };
});
