import Bottle from "bottlejs";
import Vuex, { Store } from "vuex";
import { SDF } from "./api/sdf";
import VueRouter from "vue-router";
import routes from "@/router/routes";
import { routeCheck } from "@/router";
import _ from "lodash";

export function bottleSetup() {
  let bottle = Bottle.pop("default");
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

export function bottleSetStore(router: VueRouter) {
  let bottle = Bottle.pop("default");
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
  Bottle.clear("default");
}
