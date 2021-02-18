import Vue from "vue";
import Vuex, { Module } from "vuex";
import { Store } from "vuex";
import { application } from "@/store/modules/application";
import { session } from "@/store/modules/session";
import { editor } from "@/store/modules/editor";
import Bottle from "bottlejs";
import _ from "lodash";
import { PartyBus } from "./api/partyBus";

export type SiVuexStore = Store<RootStore>;

Vue.use(Vuex);

let debug = process.env.NODE_ENV !== "production";

export interface RootStore {
  version: string;
}

export const storeData = {
  state: {
    version: "1",
  },
  modules: {
    application,
    session,
    editor,
    attribute: { namespaced: true },
    schematic: { namespaced: true },
    statusBar: { namespaced: true },
    applicationContext: { namespaced: true },
  },
  strict: debug,
};

export function instanceMapState<T>(...paths: string[]): T | null {
  const bottle = Bottle.pop("default");
  return _.get(bottle.container.Store.state, paths, null);
}

export function ctxMapState<T>(
  ctx: InstanceStoreContext<any>,
  ...paths: string[]
): T {
  const bottle = Bottle.pop("default");
  let searchPath = [ctx.storeName, ctx.instanceId];
  if (paths.length > 0) {
    searchPath = searchPath.concat(paths);
  }
  return _.get(bottle.container.Store.state, searchPath);
}

export interface IInstanceStoreContext {
  storeName: string;
  componentId: string;
  instanceId: string;
}

export class InstanceStoreContext<State> {
  storeName: IInstanceStoreContext["storeName"];
  componentId: IInstanceStoreContext["componentId"];
  instanceId: IInstanceStoreContext["instanceId"];

  constructor(ctx: IInstanceStoreContext) {
    this.storeName = ctx.storeName;
    this.componentId = ctx.componentId;
    this.instanceId = ctx.instanceId;
  }

  name(): string {
    return `${this.componentId}-${this.instanceId}`;
  }

  dispatchPath(...pathParts: string[]): string {
    let path = `${this.storeName}/${this.instanceId}`;
    if (pathParts.length > 0) {
      const extraPath = pathParts.join("/");
      path = `${path}/${extraPath}`;
    }
    return path;
  }

  async dispatch(path: string, payload?: any): Promise<any> {
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;
    let reply = await store.dispatch(this.dispatchPath(path), payload);
    return reply;
  }

  get state(): State {
    let bottle = Bottle.pop("default");
    let store = bottle.container.Store;
    return store.state[this.storeName][this.instanceId];
  }
}

export async function registerStore(
  ctx: InstanceStoreContext<any>,
  moduleData: Module<any, any>,
  events?: { eventName: () => string }[],
) {
  const bottle = Bottle.pop("default");
  const store: SiVuexStore = bottle.container.Store;
  if (
    store.hasModule(ctx.storeName) &&
    !store.hasModule([ctx.storeName, ctx.instanceId])
  ) {
    store.registerModule([ctx.storeName, ctx.instanceId], moduleData);
  }
  if (events && events.length) {
    let partyBus: PartyBus = bottle.container.PartyBus;
    partyBus.subscribeToEvents(ctx.storeName, ctx.instanceId, events);
  }
}

export function unregisterStore(ctx: InstanceStoreContext<any>) {
  const bottle = Bottle.pop("default");
  const store: SiVuexStore = bottle.container.Store;
  if (store.hasModule([ctx.storeName, ctx.instanceId])) {
    store.unregisterModule([ctx.storeName, ctx.instanceId]);
  }
}
