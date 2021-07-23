import Bottle from "bottlejs";
import { SDF } from "./api/sdf";
import { Config } from "./config";
import VueRouter from "vue-router";
import routes from "@/router/routes";
import { routeCheck } from "@/router";
import _ from "lodash";

export function bottleSetup(config: Config) {
  let bottle = Bottle.pop("default");
  bottle.factory("SDF", function(_container): SDF {
    return new SDF(config);
  });
  bottle.factory("Router", function(_container): VueRouter {
    const router = new VueRouter({
      mode: "history",
      base: config.routerBase,
      routes,
    });
    router.beforeEach(async (to, from, next) => {
      await routeCheck(to, from, next);
    });
    return router;
  });
}

export function bottleSetStore(router: VueRouter, config: Config) {
  let bottle = Bottle.pop("default");
  bottle.factory("SDF", function(_container): SDF {
    return new SDF(config);
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
