import _ from 'lodash';
import { ulid } from "ulidx";
import * as Auth0 from 'auth0';

import { createWorkspace } from "./workspaces.service";

export type UserId = string;

export type User = {
  id: string; // our id
  externalId: string; // auth0 id - based on 3rd party
  externalDetails?: any; // json blob, just store auth0 details for now
  nickname: string;
  firstName?: string;
  lastName?: string;
  email?: string;
  emailVerified: boolean;
  pictureUrl?: string;
  needsTosUpdate?: boolean;
};

const usersById: Record<UserId, User> = {};

export async function getUserById(id: UserId) {
  return _.find(_.values(usersById), (u) => u.id === id);
}
export async function findUserByExternalId(externalId: string) {
  return _.find(_.values(usersById), (u) => u.externalId === externalId);
}

export async function createOrUpdateUserFromAuth0Details(auth0UserData: Auth0.UserData) {
  // auth0 docs showing user_id, but looks like "sub" contains the identifier
  // TODO: check data when logging in with other providers
  const externalId = auth0UserData.user_id || (auth0UserData as any).sub;

  const existingUser = await findUserByExternalId(externalId);

  const user: User = {
    id: existingUser?.id || ulid(),
    externalId,
    externalDetails: auth0UserData,
    nickname: auth0UserData.nickname || auth0UserData.given_name || auth0UserData.email || 'user',
    firstName: auth0UserData.given_name,
    lastName: auth0UserData.family_name,
    emailVerified: auth0UserData.email_verified || false,
    pictureUrl: auth0UserData.picture,
  };

  usersById[user.id] = user;

  // for new users we'll want to create a default workspace, do some tracking, etc...
  if (!existingUser) {
    await createWorkspace(user);
    // TODO: probably fire off some info to posthog
  }

  return user;
}
export async function updateUser(user: User) {
  usersById[user.id] = _.omit(user, 'needsTosUpdate');
}
