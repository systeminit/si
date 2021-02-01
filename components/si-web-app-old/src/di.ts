import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";
import Vuex, { Store } from "vuex";
import { RootStore } from "./store";

export function bottleSetup(storeData: any) {
  let bottle = Bottle.pop("default");
  bottle.factory("Store", function(_container): Store<RootStore> {
    return new Vuex.Store(storeData);
  });
  bottle.service("SDF", SDF);
}

export function bottleSetStore(store: any) {
  let bottle = Bottle.pop("default");
  bottle.factory("Store", function(_container): Store<RootStore> {
    return store;
  });
  bottle.service("SDF", SDF);
}

export function bottleClear() {
  Bottle.clear();
}
