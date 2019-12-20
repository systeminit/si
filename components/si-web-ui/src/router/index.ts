import Vue from "vue";
import VueRouter from "vue-router";
import Home from "@/views/Home.vue";
import SignUp from "@/views/SignUp.vue";
import SignIn from "@/views/SignIn.vue";
import NotFound from "@/views/NotFound.vue";

import { auth } from "@/auth";

Vue.use(VueRouter);

const routes = [
  {
    path: "/",
    name: "home",
    component: Home,
  },
  {
    path: "/signin",
    name: "signin",
    component: SignIn,
  },
  {
    path: "/signup",
    name: "signup",
    component: SignUp,
  },
  {
    path: "/about",
    name: "about",
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () =>
      import(/* webpackChunkName: "about" */ "../views/About.vue"),
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
  if (
    (await auth.isAuthenticated()) ||
    to.path == "/signin" ||
    to.path == "/signup"
  ) {
    return next();
  } else {
    next("/signin");
  }
});

export default router;
