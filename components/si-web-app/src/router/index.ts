import Vue from "vue";
import VueRouter from "vue-router";

import SignIn from "@/views/SignIn.vue";
import NotFound from "@/views/NotFound.vue";

import { auth } from "@/auth";

Vue.use(VueRouter);

const routes = [
  {
    path: "/signin",
    name: "signin",
    component: SignIn,
  },
  {
    path: "*",
    component: NotFound,
  },
];

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
