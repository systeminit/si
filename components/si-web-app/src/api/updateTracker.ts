import Bottle from "bottlejs";
import { SiVuexStore } from "@/store";

export interface IStorePaths {
  [key: string]: Set<string>;
}

export class UpdateTracker {
  storePaths: IStorePaths;

  constructor() {
    this.storePaths = {};
  }

  register(objectType: string, storePath: string) {
    if (!this.storePaths[objectType]) {
      this.storePaths[objectType] = new Set();
    }
    this.storePaths[objectType].add(storePath);
  }

  unregister(objectType: string, storePath: string) {
    if (!this.storePaths[objectType]) {
      this.storePaths[objectType] = new Set();
    }
    this.storePaths[objectType].delete(storePath);
    if (this.storePaths[objectType].size == 0) {
      delete this.storePaths[objectType];
    }
  }

  async dispatch(objectType: string, payload: any) {
    let bottle = Bottle.pop("default");
    let store: SiVuexStore = bottle.container.Store;
    if (this.storePaths[objectType]) {
      for (const storePath of this.storePaths[objectType].values()) {
        store.dispatch(`${storePath}/from${objectType}`, payload);
      }
    }
  }
}
