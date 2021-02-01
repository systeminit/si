import Bottle from "bottlejs";
import Vuex, { Store } from "vuex";
import { RootStore } from "./store";
import { SDF } from "./api/sdf";
import VueRouter from "vue-router";
import routes from "@/router/routes";
import { routeCheck } from "@/router";

export function bottleSetup(storeData: any) {
  let bottle = Bottle.pop("default");
  bottle.factory("Store", function(_container): Store<RootStore> {
    return new Vuex.Store(storeData);
  });
  bottle.factory("SDF", function(_container): SDF {
    return new SDF();
  });
  bottle.factory("Router", function(_container): VueRouter {
    const router = new VueRouter({
      mode: "history",
      base: process.env.BASE_URL,
      routes,
    });
    router.beforeEach(async (to, from, next) => {
      await routeCheck(to, from, next);
    });
    return router;
  });
}

export function bottleSetStore(store: any, router: VueRouter) {
  let bottle = Bottle.pop("default");
  bottle.factory("Store", function(_container): Store<RootStore> {
    return store;
  });
  bottle.factory("SDF", function(_container): SDF {
    return new SDF();
  });
  bottle.factory("Router", function(_container): VueRouter {
    router.beforeEach(async (to, from, next) => {
      await routeCheck(to, from, next);
    });
    return router;
  });
}

export function bottleClear() {
  Bottle.clear();
}
