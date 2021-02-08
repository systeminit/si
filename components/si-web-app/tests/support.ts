import {
  uniqueNamesGenerator,
  adjectives,
  colors,
  animals,
} from "unique-names-generator";
import { SignupDal, ISignupDalReply } from "@/api/sdf/dal/signupDal";
import { SessionDal, IGetDefaultsReply } from "@/api/sdf/dal/sessionDal";
import { User } from "@/api/sdf/model/user";
import { BillingAccount } from "@/api/sdf/model/billingAccount";
import { Entity } from "@/api/sdf/model/entity";
import Bottle from "bottlejs";
import { IApplicationListEntry } from "@/store/modules/application";
import { ISetDefaultsReply } from "@/store/modules/session";

export function createFakeName(): string {
  const randomName: string = uniqueNamesGenerator({
    dictionaries: [adjectives, colors, animals],
  });
  return randomName;
}

export interface INewBillingAccount {
  billingAccount: BillingAccount;
  user: User;
}

export async function createBillingAccountAndLogin(): Promise<
  INewBillingAccount
> {
  const billingAccountName = createFakeName();

  const reply = await SignupDal.createBillingAccount({
    billingAccountName,
    billingAccountDescription: "acme",
    userName: "a",
    userEmail: "a",
    userPassword: "a",
  });
  if (reply.error) {
    throw new Error(reply.error.message);
  }

  const loginReply = await SessionDal.login({
    billingAccountName,
    userEmail: "a",
    userPassword: "a",
  });
  if (loginReply.error) {
    throw new Error(loginReply.error.message);
  }

  return { billingAccount: loginReply.billingAccount, user: loginReply.user };
}

export interface IApplication {
  application: Entity;
}

export async function setSessionDefaults(): Promise<ISetDefaultsReply> {
  let bottle = Bottle.pop("default");
  let store = bottle.container.Store;
  return await store.dispatch("session/setDefaults");
}

export async function createApplication(): Promise<Entity> {
  let bottle = Bottle.pop("default");
  let store = bottle.container.Store;
  let applicationName = createFakeName();
  let currentWorkspace = store.state.session.currentWorkspace;
  let currentSystem = store.state.session.currentSystem;
  let reply = await store.dispatch("application/createApplication", {
    applicationName,
    workspaceId: currentWorkspace.id,
    systemId: currentSystem.id,
  });
  return reply.application;
}

export async function createApplicationListEntry(): Promise<
  IApplicationListEntry
> {
  let bottle = Bottle.pop("default");
  let store = bottle.container.Store;
  let applicationName = createFakeName();
  let currentWorkspace = store.state.session.currentWorkspace;
  let currentSystem = store.state.session.currentSystem;
  let reply = await store.dispatch("application/createApplication", {
    applicationName,
    workspaceId: currentWorkspace.id,
    systemId: currentSystem.id,
  });
  return reply;
}
