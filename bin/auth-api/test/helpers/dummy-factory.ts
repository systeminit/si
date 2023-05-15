import { Prisma, PrismaClient, User } from '@prisma/client';
import { ulid } from 'ulidx';
import _ from 'lodash';
import { saveTosAgreement, LATEST_TOS_VERSION_ID } from '../../src/services/tos.service';
import { createWorkspace } from '../../src/services/workspaces.service';
import { AuthProviders } from '../../src/services/auth.service';

const prisma = new PrismaClient();

// helper to override only keys that already existed in the defaults
function assignOverrides<T extends { [k: string]: any }>(defaults: T, overrides?: { [j: string]: any }): T {
  if (!overrides) return defaults;

  const obj = _.cloneDeep(defaults);
  _.each(obj, (val, key) => {
    if (Object.hasOwn(overrides, key)) {
      (obj as any)[key] = overrides[key];
    }
  });
  return obj;
}

let dummyUserCounter = +new Date();
export async function createDummyUser(options?: {
  user?: Partial<User> & {
    authProvider?: AuthProviders,
  },
  tos?: boolean,
  workspace?: boolean,
}) {
  const counter = dummyUserCounter++;
  const provider = options?.user?.authProvider || 'google';
  const userData = assignOverrides({
    id: ulid(),
    auth0Details: {} as Prisma.JsonObject,
    auth0Id: `${provider}|${counter}`,
    nickname: `Dummy${counter}`,
    firstName: `First${counter}`,
    lastName: `Last${counter}`,
    email: `dummy${counter}@systeminit.dev`,
    emailVerified: true,
    // pictureUrl:
    githubUsername: `githubuser${counter}`,
    discordUsername: `discord${counter}#1234`,
  }, options?.user);

  const user = await prisma.user.create({ data: userData });

  if (options?.tos !== false) {
    await saveTosAgreement(user, LATEST_TOS_VERSION_ID, '1.2.3.4');
  }

  const workspace = await createWorkspace(user);

  return { user, workspace };
}
