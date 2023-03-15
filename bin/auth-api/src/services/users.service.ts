import _ from 'lodash';
import { ulid } from "ulidx";
import * as Auth0 from 'auth0';
import { Prisma, PrismaClient } from '@prisma/client';

import { createWorkspace } from "./workspaces.service";
import { LATEST_TOS_VERSION_ID } from './tos.service';

const prisma = new PrismaClient();

export type UserId = string;

// export type User = {
//   id: string; // our id
//   auth0Id: string; // auth0 id - based on 3rd party
//   auth0Details?: any; // json blob, just store auth0 details for now
//   nickname: string;
//   firstName?: string;
//   lastName?: string;
//   email: string;
//   emailVerified: boolean;
//   pictureUrl?: string;

//   needsTosUpdate?: boolean;
// };

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

  const agreedTosVersion = userWithTosAgreement?.TosAgreement?.[0]?.tosVersionId;
  const needsTosUpdate = !agreedTosVersion || agreedTosVersion < LATEST_TOS_VERSION_ID;

  return {
    ..._.omit(userWithTosAgreement, 'TosAgreement'),
    agreedTosVersion,
    needsTosUpdate,
  };
}
export type UserWithTosStatus = Awaited<ReturnType<typeof getUserById>>;

export async function getUserByAuth0Id(auth0Id: string) {
  return await prisma.user.findUnique({ where: { auth0Id } });
}

export async function createOrUpdateUserFromAuth0Details(auth0UserData: Auth0.UserData) {
  // auth0 docs showing user_id, but looks like "sub" contains the identifier
  // TODO: check data when logging in with other providers
  const auth0Id = auth0UserData.user_id || (auth0UserData as any).sub;

  const existingUser = await getUserByAuth0Id(auth0Id);

  const userData = {
    // TODO: figure out json fields...
    auth0Details: auth0UserData as Prisma.JsonObject,
    nickname: auth0UserData.nickname || auth0UserData.given_name || auth0UserData.email || 'user',
    firstName: auth0UserData.given_name,
    lastName: auth0UserData.family_name,
    // need to confirm email will always be present with our chosen auth providers
    email: auth0UserData.email!,
    emailVerified: auth0UserData.email_verified || false,
    pictureUrl: auth0UserData.picture,
  };

  if (existingUser) {
    _.assign(existingUser, userData);
    await prisma.user.update({
      where: { id: existingUser.id },
      data: _.omit(existingUser, 'id', 'auth0Id', 'auth0Details'),
    });
    return existingUser;
  } else {
    const newUser = await prisma.user.create({
      data: {
        id: ulid(),
        auth0Id,
        ...userData,
      },
    });
    // for new users we'll want to create a default workspace, do some tracking, etc...
    await createWorkspace(newUser);
    // TODO: probably fire off some info to posthog
    return newUser;
  }
}

// const updateUserData = Prisma.validator<Prisma.UserArgs>()({
//   select: { email: true, name: true },
// })
// export async function updateUser(
//   userId: UserId,
//   data:
// ) {

//   usersById[user.id] = _.omit(user, 'needsTosUpdate');
// }
