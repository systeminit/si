import _ from "lodash";
import { ulid } from "ulidx";
import * as Auth0 from "auth0";
import { InstanceEnvType, Prisma, PrismaClient } from "@prisma/client";

import { createWorkspace, SAAS_WORKSPACE_URL, LOCAL_WORKSPACE_URL } from "./workspaces.service";
import { tracker } from "../lib/tracker";
import { fetchAuth0Profile } from "./auth0.service";
import { isLocalAuth } from "./auth0-local.service";
import { ApiError } from "../lib/api-error";
import { findLatestTosForUser } from "./tos.service";
import {
  createCustomer,
  createPaidSubscription,
  createTrialSubscription,
} from "../lib/lago";

const prisma = new PrismaClient();

export type UserId = string;
export type Auth0User = Auth0.GetUsers200ResponseOneOfInner;
export type ApiResponse<T> = Auth0.ApiResponse<T>;

export async function getUserById(id: UserId): Promise<any> {
  const userWithTosAgreement = await prisma.user.findUnique({
    where: { id },
    include: {
      TosAgreement: {
        orderBy: {
          id: "desc",
        },
        select: {
          tosVersionId: true,
        },
        take: 1,
      },
    },
  });
  if (!userWithTosAgreement) return null;

  const agreedTosVersion =
    userWithTosAgreement?.TosAgreement?.[0]?.tosVersionId;

  const latestTosVersion = await findLatestTosForUser(userWithTosAgreement);

  const needsTosUpdate =
    !agreedTosVersion || agreedTosVersion < latestTosVersion;

  return {
    ..._.omit(userWithTosAgreement, "TosAgreement"),
    agreedTosVersion,
    needsTosUpdate,
  };
}

export type User = NonNullable<
  Awaited<ReturnType<typeof prisma.user.findUnique>>
>;
export type UserWithTosStatus = NonNullable<
  Awaited<ReturnType<typeof getUserById>>
>;

export async function getUserByAuth0Id(auth0Id: string) {
  return prisma.user.findUnique({ where: { auth0Id } });
}

export async function getUserByEmail(email: string) {
  return prisma.user.findFirst({ where: { email } });
}

export async function getUsersByEmail(email: string) {
  return prisma.user.findMany({ where: { email } });
}

export async function createInvitedUser(email: string) {
  return await prisma.user.create({
    data: {
      id: ulid(),
      email,
    },
  });
}

export async function getSuspendedUsers() {
  const suspendedUsers = await prisma.user.findMany({
    where: {
      suspendedAt: {
        not: null,
      },
    },
  });

  return suspendedUsers;
}

export async function getQuarantinedUsers() {
  const quarantinedUsers = await prisma.user.findMany({
    where: {
      quarantinedAt: {
        not: null,
      },
    },
  });

  return quarantinedUsers;
}

export async function createOrUpdateUserFromAuth0Details(
  auth0UserDataRaw: ApiResponse<Auth0User>,
) {
  const auth0UserData = auth0UserDataRaw.data;
  // not sure why this type says the id could be empty? probably will not happen but we'll watch for this error
  if (!auth0UserData.user_id) throw new Error("Missing auth0 user_id");
  if (!auth0UserData.email) throw new Error("Missing auth0 email");

  const auth0Id = auth0UserData.user_id;

  let user = await getUserByAuth0Id(auth0Id);
  // if no user found, we'll check if there is an invited (but not yet signed up) user with a matching email
  // TODO: currently we can have multiple users with the same email, and we are just grabbing the first
  // we'll also want to switch to link accounts rather than creating anothe one
  if (!user) {
    user = await getUserByEmail(auth0UserData.email);
    if (user?.signupAt) user = null;
  }

  let isSignup = false;
  if (user) {
    if (!user.signupAt) {
      user.auth0Id ||= auth0Id;
      user.signupAt ||= new Date();
      isSignup = true;
    }

    setUserDataFromAuth0Details(user, auth0UserData, isSignup);

    // LOCAL AUTH MODE: Mark onboarding as complete for ALL local users
    // (not just signups - in case user already exists from previous run)
    if (isLocalAuth()) {
      user.onboardingDetails = {
        ...(typeof user.onboardingDetails === 'object' ? user.onboardingDetails : {}),
        reviewedProfile: new Date().toISOString(),
        firstTimeModal: false, // Skip the onboarding modal
      } as Prisma.JsonObject;

      // eslint-disable-next-line no-console
      console.log(JSON.stringify({
        timestamp: new Date().toISOString(),
        level: "info",
        type: "local-auth",
        action: "skip_onboarding",
        userId: user.id,
        isSignup,
        message: "🔧 LOCAL AUTH MODE: Marking onboarding as complete for local user",
      }));
    }

    await prisma.user.update({
      where: { id: user.id },
      data: {
        ..._.omit(user, "id", "onboardingDetails"),
        auth0Details: auth0UserData as Prisma.JsonObject,
        ...(user.onboardingDetails && { onboardingDetails: user.onboardingDetails }),
      },
    });
    tracker.identifyUser(user);
  } else {
    isSignup = true;
    const userData = setUserDataFromAuth0Details({}, auth0UserData, isSignup);

    // LOCAL AUTH MODE: Mark onboarding as complete
    if (isLocalAuth()) {
      userData.onboardingDetails = {
        reviewedProfile: new Date().toISOString(),
        firstTimeModal: false, // Skip the onboarding modal
      } as Prisma.JsonObject;

      // eslint-disable-next-line no-console
      console.log(JSON.stringify({
        timestamp: new Date().toISOString(),
        level: "info",
        type: "local-auth",
        action: "skip_onboarding",
        message: "🔧 LOCAL AUTH MODE: Marking onboarding as complete for local user",
      }));
    }

    user = await prisma.user.create({
      data: {
        id: ulid(),
        signupAt: new Date(),
        auth0Id,
        ...userData,
      },
    });

    tracker.identifyUser(user);
  }

  if (isSignup) {
    tracker.trackEvent(user, "auth_connected", {
      id: user.id,
      email: user.email,
      firstName: user.firstName,
      lastName: user.lastName,
    });

    // LOCAL AUTH MODE: Create local development workspace
    if (isLocalAuth()) {
      // eslint-disable-next-line no-console
      console.log(JSON.stringify({
        timestamp: new Date().toISOString(),
        level: "info",
        type: "local-auth",
        action: "create_workspace",
        userId: user.id,
        workspaceName: "Local Development",
        workspaceUrl: LOCAL_WORKSPACE_URL,
        message: "🔧 LOCAL AUTH MODE: Auto-creating local development workspace",
      }));

      await createWorkspace(
        user,
        InstanceEnvType.LOCAL,
        LOCAL_WORKSPACE_URL,
        "Local Development",
        true,
        "",
      );
    } else {
      // PRODUCTION MODE: Create SaaS production workspace
      await createWorkspace(
        user,
        InstanceEnvType.SI,
        SAAS_WORKSPACE_URL,
        `${user.nickname}'s  Production Workspace`,
        true,
        "",
      );
    }
  }

  return user;
}

export async function refreshUserAuth0Profile(user: User) {
  if (!user.auth0Id) {
    throw new ApiError("Conflict", "User has no auth0 id");
  }
  const auth0Details = await fetchAuth0Profile(user.auth0Id);
  setUserDataFromAuth0Details(user, auth0Details.data);
  await saveUser(user);
}

function setUserDataFromAuth0Details(
  user: any,
  auth0Details: Auth0User,
  isSignup = false,
) {
  // save most up to date copy of auth0 details
  user.auth0Details = auth0Details as Prisma.JsonObject;

  // fill in any empty data we can infer from auth0 data
  // pickBy just removed empty values
  _.each(
    {
      nickname:
        auth0Details.nickname || auth0Details.given_name || auth0Details.email,
      firstName: auth0Details.given_name,
      lastName: auth0Details.family_name,
      email: auth0Details.email,
      // Coerce email_verified to a boolean in case it's anything else
      // thanks to Auth0 and mapping SAML assertions
      emailVerified: !!auth0Details.email_verified,
      pictureUrl: auth0Details.picture,
      // fairly certain nickname is github username when auth provider is github
      ...(auth0Details.user_id?.startsWith("github|") && {
        githubUsername: auth0Details.nickname,
      }),
    },
    (val, key) => {
      // special handling to leave a photo that the user explicitly cleared as empty
      // TODO: ideally we'd have some other way of knowing the user explicitly cleared it, but this should work
      if (key === "pictureUrl" && !isSignup && !user.pictureUrl) return;
      if (!user[key]) user[key] = val;
    },
  );
  return user;
}

export async function getUserSignupReport(startDate: Date, endDate: Date) {
  const startOfDay = new Date(startDate.setHours(0, 0, 0, 0));
  const endOfDay = new Date(endDate.setHours(23, 59, 59, 999));
  const signups = await prisma.user.findMany({
    where: {
      signupAt: {
        lte: endOfDay,
        gt: startOfDay,
      },
      NOT: {
        signupAt: null,
      },
      AND: {
        emailVerified: true,
      },
    },
    select: {
      id: true,
      firstName: true,
      email: true,
      discordUsername: true,
      githubUsername: true,
      lastName: true,
      auth0Id: true,
      signupAt: true,
    },
    orderBy: {
      signupAt: "desc",
    },
  });

  return signups;
}

export async function saveUser(user: User) {
  await prisma.user.update({
    where: { id: user.id },
    data: {
      ..._.omit(
        user,
        "id",
        "auth0Id",
        "auth0Details",
        "needsTosUpdate",
        "agreedTosVersion",
        "onboardingDetails",
      ),
      // this is dumb... prisma is annoying
      onboardingDetails:
        (user.onboardingDetails as Prisma.JsonObject) || undefined,
      auth0Details: (user.auth0Details as Prisma.JsonObject) || undefined,
    },
  });
  tracker.identifyUser(user);
  return user;
}

export async function create_lago_customer_records(user: User) {
  await createCustomer(
    user.id,
    user.firstName || "",
    user.lastName || "",
    user.email,
  );
  tracker.trackEvent(user, "created_lago_customer", {
    userPk: user.id,
    createdAt: new Date(),
  });
  await createTrialSubscription(user.id);
  tracker.trackEvent(user, "created_lago_trial_subscription", {
    userPk: user.id,
    createdAt: new Date(),
    plan: "launch_trial",
  });
  await createPaidSubscription(user.id);
  tracker.trackEvent(user, "created_lago_payg_subscription", {
    userPk: user.id,
    createdAt: new Date(),
    plan: "launch_pay_as_you_go",
  });
}
