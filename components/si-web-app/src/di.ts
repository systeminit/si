import Bottle from "bottlejs";
import Vuex, { Store } from "vuex";
import { RootStore } from "./store";
import { SDF } from "./api/sdf";
import VueRouter from "vue-router";
import routes from "@/router/routes";
import { routeCheck } from "@/router";
import _ from "lodash";
import { UpdateTracker } from "./api/updateTracker";
import { PartyBus } from "./api/partyBus";
import { Persister } from "./api/persister";

export function bottleSetup(storeData: any) {
  let bottle = Bottle.pop("default");
  bottle.factory("Store", function(_container): Store<RootStore> {
    return new Vuex.Store(_.cloneDeep(storeData));
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
  bottle.factory("UpdateTracker", function(_container): UpdateTracker {
    return new UpdateTracker();
  });
  bottle.factory("PartyBus", function(_container): PartyBus {
    return new PartyBus();
  });
  bottle.factory("Persister", function(_container): Persister {
    return new Persister();
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
  bottle.factory("UpdateTracker", function(_container): UpdateTracker {
    return new UpdateTracker();
  });
  bottle.factory("PartyBus", function(_container): PartyBus {
    return new PartyBus();
  });
  bottle.factory("Persister", function(_container): Persister {
    return new Persister();
  });
}

export function bottleClear() {
  Bottle.clear("default");
}
