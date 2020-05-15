import Vue from "vue";
import VueRouter from "vue-router";

// @ts-ignore: Unreachable code error
import routes from "./routes";

import { auth } from "@/auth";

Vue.use(VueRouter);

const router = new VueRouter({
  mode: "history",
  base: process.env.BASE_URL,
  routes,
});

router.beforeEach(async (to, _from, next) => {
  if ((await auth.isAuthenticated()) || to.path == "/signin") {
    return next();
  } else {
    next("/signin");
  }
});

export default router;