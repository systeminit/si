import Vue from "vue";
import VueRouter from "vue-router";

// @ts-ignore: Unreachable code error
import routes from "./routes";

import store from "@/store";
import _ from "lodash";

Vue.use(VueRouter);

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

const routeCheck = async (to, from, next) => {
  //const span = telemetry.routeSpan(`web.router ${to.path}`);
  //span.setAttributes({
  //  "web.route.to.path": to.path,
  //  "web.route.to.full_path": to.fullPath,
  //  "web.route.to.params": to.params,
  //  "web.route.to.query": to.query,
  //  "web.route.to.redirected_from": to.redirectedFrom,
  //  "web.route.from.path": from.path,
  //  "web.route.from.full_path": from.fullPath,
  //  "web.route.from.params": from.params,
  //  "web.route.from.query": from.query,
  //  "web.route.from.redirected_from": from.redirectedFrom,
  //});

  // 1. Is the user asking to sign in? If so, route there.
  // 2. Is the user authenticated? If not, route there.
  // 3. Is the system loaded? If not, route to the loading screen, then redirect to the URL the user asked for.
  // 4. Route to the requested location.
  //
  if (to.path == "/signup") {
    console.log("going to signup");
    return next();
  }

  if (to.path == "/signin") {
    console.log("going to signin");
    return next();
  }

  let authenticated = await store.dispatch("user/isAuthenticated");

  if (!authenticated) {
    //  span.setAttribute("web.route.to.redirected", "/signin");
    return next("/signin");
  }

  return next();

  //if (store.state.loader.loaded) {
  //  return next();
  //} else {
  //  if (to.path == "/loading") {
  //    return next();
  //  } else {
  //    console.log("to", { to });
  //    store.commit("loader/nextUp", {
  //      name: _.cloneDeep(to.name),
  //      params: _.cloneDeep(to.params),
  //      query: _.cloneDeep(to.query),
  //    });
  //    return next("/loading");
  //  }
  //}
};

router.beforeEach(async (to, from, next) => {
  await routeCheck(to, from, next);
});

export default router;
