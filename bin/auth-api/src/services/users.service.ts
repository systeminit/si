import _ from 'lodash';
import { ulid } from "ulidx";
import * as Auth0 from 'auth0';
import { Prisma, PrismaClient } from '@prisma/client';

import { createWorkspace } from "./workspaces.service";
import { LATEST_TOS_VERSION_ID } from './tos.service';
import { tracker } from '../lib/tracker';

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

export async function createOrUpdateUserFromAuth0Details(auth0UserData: Auth0.UserData) {
  // auth0 docs showing user_id, but looks like "sub" contains the identifier
  // TODO: check data when logging in with other providers
  const auth0Id = auth0UserData.user_id || (auth0UserData as any).sub;
  const email = auth0UserData.email;

  // We want to ensure we track that we need to create a workspace for a partially
  // signed up (i.e. invited) user.
  let requiresWorkspace = false;

  let existingUser = await getUserByAuth0Id(auth0Id);
  if (!existingUser) {
    // We should check that we have a user that is potentially partially signed up
    // a partially signed up user is a user that we have a record for but that record
    // only has an email and not anything else
    const potentialUser = await getUserByEmail(email!);
    if (potentialUser && !potentialUser.signupAt) {
      existingUser = potentialUser;
      requiresWorkspace = true;
    }
  }

  const userData = {
    // TODO: figure out json fields...
    auth0Details: auth0UserData as Prisma.JsonObject,
    nickname: auth0UserData.nickname || auth0UserData.given_name || auth0UserData.email || 'user',
    firstName: auth0UserData.given_name,
    lastName: auth0UserData.family_name,
    // need to confirm email will always be present with our chosen auth providers
    email: auth0UserData.email,
    emailVerified: auth0UserData.email_verified || false,
    pictureUrl: auth0UserData.picture,

    // fairly certain nickname is github username
    ...auth0Id.startsWith('github|') && {
      githubUsername: auth0UserData.nickname,
    },
  };

  if (existingUser) {
    if (!existingUser.signupAt) {
      existingUser.signupAt = new Date();
    }
    _.assign(existingUser, userData);
    await prisma.user.update({
      where: { id: existingUser.id },
      data: _.omit(existingUser, 'id', 'auth0Id', 'auth0Details', 'onboardingDetails'),
    });

    tracker.identifyUser(existingUser);

    if (requiresWorkspace) {
      await createWorkspace(existingUser);
    }
    return existingUser;
  } else {
    const newUser = await prisma.user.create({
      data: {
        id: ulid(),
        signupAt: new Date(),
        auth0Id,
        ...userData,
      },
    });

    tracker.identifyUser(newUser);
    tracker.trackEvent(newUser, 'auth_connected', {
      id: newUser.id,
      email: newUser.email,
      firstName: newUser.firstName,
      lastName: newUser.lastName,
    });

    // user is new, so we create a default dev workspace
    await createWorkspace(newUser);

    return newUser;
  }
}

export async function saveUser(user: UserWithTosStatus) {
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
    },
  });
  tracker.identifyUser(user);
  return user;
}
