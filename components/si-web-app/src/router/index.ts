import Vue from "vue";
import VueRouter from "vue-router";

// @ts-ignore: Unreachable code error
import routes from "./routes";

import store from "@/store";
import { telemetry } from "@/utils/telemetry";

Vue.use(VueRouter);

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

router.beforeEach(async (to, from, next) => {
  const span = telemetry.routeSpan(`web.router ${to.path}`);
  span.setAttributes({
    "web.route.to.path": to.path,
    "web.route.to.full_path": to.fullPath,
    "web.route.to.params": to.params,
    "web.route.to.query": to.query,
    "web.route.to.redirected_from": to.redirectedFrom,
    "web.route.from.path": from.path,
    "web.route.from.full_path": from.fullPath,
    "web.route.from.params": from.params,
    "web.route.from.query": from.query,
    "web.route.from.redirected_from": from.redirectedFrom,
  });
  if ((await store.dispatch("user/isAuthenticated")) || to.path == "/signin") {
    return next();
  } else {
    span.setAttribute("web.route.to.redirected", "/signin");
    next("/signin");
  }
});

export default router;
