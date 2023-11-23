import _ from 'lodash';
import { ulid } from "ulidx";
import * as Auth0 from 'auth0';
import { Prisma, PrismaClient } from '@prisma/client';

import { createWorkspace } from "./workspaces.service";
import { LATEST_TOS_VERSION_ID } from './tos.service';
import { tracker } from '../lib/tracker';
import { fetchAuth0Profile } from './auth0.service';
import { ApiError } from '../lib/api-error';

const prisma = new PrismaClient();

export type UserId = string;

export async function getUserById(id: UserId) {
  const userWithTosAgreement = await prisma.user.findUnique({
    where: { id },
    include: {
      TosAgreement: {
        orderBy: {
          id: 'desc',
        },
        select: {
          tosVersionId: true,
        },
        take: 1,
      },
    },
  });
  if (!userWithTosAgreement) return null;

  const agreedTosVersion = userWithTosAgreement?.TosAgreement?.[0]?.tosVersionId;
  const needsTosUpdate = !agreedTosVersion || agreedTosVersion < LATEST_TOS_VERSION_ID;

  return {
    ..._.omit(userWithTosAgreement, 'TosAgreement'),
    agreedTosVersion,
    needsTosUpdate,
  };
}
export type User = NonNullable<Awaited<ReturnType<typeof prisma.user.findUnique>>>;
export type UserWithTosStatus = NonNullable<Awaited<ReturnType<typeof getUserById>>>;

export async function getUserByAuth0Id(auth0Id: string) {
  return prisma.user.findUnique({ where: { auth0Id } });
}

export async function getUserByEmail(email: string) {
  return prisma.user.findFirst({ where: { email } });
}

export async function createInvitedUser(email: string) {
  return await prisma.user.create({
    data: {
      id: ulid(),
      email,
    },
  });
}

export async function createOrUpdateUserFromAuth0Details(auth0UserData: Auth0.User) {
  // not sure why this type says the id could be empty? probably will not happen but we'll watch for this error
  if (!auth0UserData.user_id) throw new Error('Missing auth0 user_id');
  if (!auth0UserData.email) throw new Error('Missing auth0 email');

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

    await prisma.user.update({
      where: { id: user.id },
      data: {
        ..._.omit(user, 'id', 'onboardingDetails'),
        auth0Details: auth0UserData as Prisma.JsonObject,
      },
    });
    tracker.identifyUser(user);
  } else {
    isSignup = true;
    user = await prisma.user.create({
      data: {
        id: ulid(),
        signupAt: new Date(),
        auth0Id,
        ...setUserDataFromAuth0Details({}, auth0UserData, isSignup),
      },
    });

    tracker.identifyUser(user);
  }

  if (isSignup) {
    tracker.trackEvent(user, 'auth_connected', {
      id: user.id,
      email: user.email,
      firstName: user.firstName,
      lastName: user.lastName,
    });

    // create a default dev workspace
    await createWorkspace(user);
  }

  return user;
}

export async function refreshUserAuth0Profile(user: User) {
  if (!user.auth0Id) {
    throw new ApiError('Conflict', 'User has no auth0 id');
  }
  const auth0Details = await fetchAuth0Profile(user.auth0Id);
  setUserDataFromAuth0Details(user, auth0Details);
  await saveUser(user);
}
function setUserDataFromAuth0Details(user: any, auth0Details: Auth0.User, isSignup = false) {
  // save most up to date copy of auth0 details
  user.auth0Details = auth0Details as Prisma.JsonObject;

  // fill in any empty data we can infer from auth0 data
  // pickBy just removed empty values
  _.each({
    nickname: auth0Details.nickname || auth0Details.given_name || auth0Details.email,
    firstName: auth0Details.given_name,
    lastName: auth0Details.family_name,
    email: auth0Details.email,
    emailVerified: auth0Details.email_verified,
    pictureUrl: auth0Details.picture,
    // fairly certain nickname is github username when auth provider is github
    ...auth0Details.user_id?.startsWith('github|') && {
      githubUsername: auth0Details.nickname,
    },
  }, (val, key) => {
    // special handling to leave a photo that the user explicitly cleared as empty
    // TODO: ideally we'd have some other way of knowing the user explicitly cleared it, but this should work
    if (key === 'pictureUrl' && !isSignup && !user.pictureUrl) return;
    if (!user[key]) user[key] = val;
  });
  return user;
}

export async function saveUser(user: User) {
  await prisma.user.update({
    where: { id: user.id },
    data: {
      ..._.omit(
        user,
        'id',
        'auth0Id',
        'auth0Details',
        'needsTosUpdate',
        'agreedTosVersion',
        'onboardingDetails',
      ),
      // this is dumb... prisma is annoying
      onboardingDetails: user.onboardingDetails as Prisma.JsonObject || undefined,
      auth0Details: user.auth0Details as Prisma.JsonObject || undefined,
    },
  });
  tracker.identifyUser(user);
  return user;
}
