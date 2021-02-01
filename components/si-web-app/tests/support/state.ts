import Vuex from "vuex";
import { Store } from "vuex";
import { RootStore, storeData } from "@/store";
import {
  uniqueNamesGenerator,
  adjectives,
  colors,
  animals,
} from "unique-names-generator";
import { IBillingAccountCreateReply } from "@/api/sdf/model/billingAccount";
import { IUserLoginReply } from "@/api/sdf/model/user";
import { Entity } from "@/api/sdf/model/entity";
import _ from "lodash";
import { ActionNodeCreate, ActionSetChangeSet } from "@/store/modules/editor";
import { Node } from "@/api/sdf/model/node";
import { ChangeSet } from "@/api/sdf/model/changeSet";

type SiVuexStore = Store<RootStore>;

export function createFakeName(): string {
  const randomName: string = uniqueNamesGenerator({
    dictionaries: [adjectives, colors, animals],
  });
  return randomName;
}

export function createStore(): SiVuexStore {
  let newData = _.cloneDeep(storeData);
  newData.state = { version: "1" };
  // @ts-ignore
  return new Vuex.Store(newData);
}

export async function createNewBillingAccount(
  store: SiVuexStore,
): Promise<IBillingAccountCreateReply> {
  const billingAccountName = createFakeName();
  const response = await store.dispatch("billingAccount/create", {
    billingAccountName,
    billingAccountDescription: billingAccountName,
    userName: "bobo",
    userEmail: "bobo@tclown.com",
    userPassword: "bobolives",
  });
  await login(store, response);
  await store.dispatch("workspace/default");
  await store.dispatch("organization/default");
  await store.dispatch("system/default");
  return response;
}

export async function login(
  store: SiVuexStore,
  nba: IBillingAccountCreateReply,
): Promise<IUserLoginReply> {
  return await store.dispatch("user/login", {
    billingAccountName: nba.billingAccount.name,
    email: "bobo@tclown.com",
    password: "bobolives",
  });
}

export async function createApplication(store: SiVuexStore): Promise<Entity> {
  const applicationName = createFakeName();
  let app_entity = await store.dispatch("application/create", {
    name: applicationName,
  });
  return app_entity;
}

export async function selectApplication(
  store: SiVuexStore,
  application: Entity,
) {
  await store.dispatch("editor/setApplication", { id: application.id });
}

export async function createChangeSet(
  store: SiVuexStore,
  name?: string,
): Promise<ChangeSet> {
  let payload = {
    name: name ? name : createFakeName(),
  };
  return await store.dispatch("editor/changeSetCreate", payload);
}

export async function selectChangeSet(
  store: SiVuexStore,
  payload: ActionSetChangeSet,
) {
  await store.dispatch("editor/setChangeSet", payload);
}

export async function createNode(
  store: SiVuexStore,
  payload: ActionNodeCreate,
): Promise<Node> {
  return store.dispatch("editor/nodeCreate", payload);
}

export async function selectNode(store: SiVuexStore, payload: Node) {
  await store.dispatch("editor/node", payload);
}
