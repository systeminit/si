import Bottle from "bottlejs";
import { SDF } from "./api/sdf";
import { Config } from "./config";
// import router from "./router";
// import { routeCheck } from "./router";
// import _ from "lodash";

export function bottleSetup(config: Config) {
  const bottle = Bottle.pop("default");
  bottle.factory("SDF", function (_container): SDF {
    return new SDF(config);
  });

  // bottle.factory("Router", function(_container): VueRouter {
  //   return router;
  // });
}

// @ts-ignore
export function bottleSetStore(router: VueRouter, config: Config) {
  const bottle = Bottle.pop("default");
  bottle.factory("SDF", function (_container): SDF {
    return new SDF(config);
  });

  // bottle.factory("Router", function(_container): VueRouter {
  //   router.beforeEach(async (to, from, next) => {
  //     await routeCheck(to, from, next);
  //   });
  //   return router;
  // });
}

export function bottleClear() {
  Bottle.clear("default");
}
