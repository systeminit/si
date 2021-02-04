import Vue from "vue";
import VueRouter from "vue-router";
import _ from "lodash";
import Bottle from "bottlejs";

Vue.use(VueRouter);

export const routeCheck = async (to: any, from: any, next: any) => {
  // 1. Is the user asking to sign in? If so, route there.
  // 2. Is the user authenticated? If not, route there.
  // 3. Is the system loaded? If not, route to the loading screen, then redirect to the URL the user asked for.
  // 4. Route to the requested location.
  //
  if (to.path == "/authenticate/signup") {
    return next();
  } else if (to.path == "/authenticate/login") {
    return next();
  }

  let bottle = Bottle.pop("default");
  let store = bottle.container.Store;
  let authenticated = await store.dispatch("session/isAuthenticated");
  if (authenticated === false || authenticated.error) {
    if (authenticated.error) {
      console.log("Error checking authentication", authenticated);
    }
    return next("/authenticate/login");
  } else {
    return next();
  }
};
