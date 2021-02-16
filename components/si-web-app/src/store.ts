import Vue from "vue";
import Vuex from "vuex";
import { Store } from "vuex";
import { application } from "@/store/modules/application";
import { session } from "@/store/modules/session";
import Bottle from "bottlejs";
import _ from "lodash";

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
  ctx: InstanceStoreContext,
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

export class InstanceStoreContext {
  storeName: IInstanceStoreContext["storeName"];
  componentId: IInstanceStoreContext["componentId"];
  instanceId: IInstanceStoreContext["instanceId"];

  constructor(ctx: IInstanceStoreContext) {
    this.storeName = ctx.storeName;
    this.componentId = ctx.componentId;
    this.instanceId = ctx.instanceId;
  }

  activateName(): string {
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
}
