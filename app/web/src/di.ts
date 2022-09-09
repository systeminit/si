import Bottle from "bottlejs";
import { SDF } from "./api/sdf";
// import router from "./router";
// import { routeCheck } from "./router";
// import _ from "lodash";

export function bottleSetup() {
  const bottle = Bottle.pop("default");
  bottle.factory("SDF", function sdfBottleFactory(_container): SDF {
    return new SDF();
  });

  // bottle.factory("Router", function(_container): VueRouter {
  //   return router;
  // });
}

// @ts-ignore
export function bottleSetStore(_router: VueRouter) {
  const bottle = Bottle.pop("default");
  bottle.factory("SDF", function sdfBottleFactory(_container): SDF {
    return new SDF();
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
