import { InstanceStoreContext } from "@/store";
import _ from "lodash";
import Bottle from "bottlejs";
import VueRouter from "vue-router";

export interface PersistanceBackend {
  setItem: (key: string, value: any) => void;
  getItem: (key: string) => any;
  removeItem: (key: string) => void;
}

export class Persister {
  storage: PersistanceBackend;

  constructor() {
    this.storage = window.sessionStorage;
  }

  setData(key: string, data: any) {
    this.storage.setItem(key, JSON.stringify(data));
  }

  getData(key: string): any {
    let jsonData = this.storage.getItem(key);
    if (jsonData) {
      let data = JSON.parse(jsonData);
      data = _.mapValues(data, v => {
        if (v && v.storeName && v.componentId && v.instanceId) {
          return new InstanceStoreContext({
            storeName: v.storeName,
            componentId: v.componentId,
            instanceId: v.instanceId,
          });
        } else {
          return v;
        }
      });
      return data;
    } else {
      return null;
    }
  }

  async updateQueryParam(payload: Record<string, any>) {
    let bottle = Bottle.pop("default");
    let router: VueRouter = bottle.container.Router;
    await router
      .replace({
        query: Object.assign({}, { ...router.currentRoute.query }, payload),
      })
      .catch(() => {});
  }

  async removeQueryParam(payload: string[]) {
    let bottle = Bottle.pop("default");
    let router: VueRouter = bottle.container.Router;
    const query = Object.assign({}, router.currentRoute.query);
    for (const param of payload) {
      delete query[param];
    }
    await router.replace({ query }).catch(() => {});
  }

  async wipeQueryParams() {
    let bottle = Bottle.pop("default");
    let router: VueRouter = bottle.container.Router;
    await router
      .replace({
        query: {},
      })
      .catch(() => {});
  }
}
