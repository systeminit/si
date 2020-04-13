import Vue from "vue";
import VueRouter from "vue-router";
import Home from "@/views/Home.vue";
import SignUp from "@/views/SignUp.vue";
import SignIn from "@/views/SignIn.vue";
import NotFound from "@/views/NotFound.vue";
import Workspace from "@/views/Workspace.vue";
import WorkspaceShowEntity from "@/views/WorkspaceShowEntity.vue";
import WorkspaceCreateEntity from "@/views/WorkspaceCreateEntity.vue";
import WorkspaceEditEntity from "@/views/WorkspaceEditEntity.vue";

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
    path: "/o/:organizationId/w/:workspaceId/show/:entityType/:entityId",
    name: "workspaceShowEntity",
    component: WorkspaceShowEntity,
    props: true,
  },
  {
    path: "/o/:organizationId/w/:workspaceId/create/:entityType",
    name: "workspaceCreateEntity",
    component: WorkspaceCreateEntity,
    props: true,
  },
  {
    path: "/o/:organizationId/w/:workspaceId/edit/:entityType/:entityId",
    name: "workspaceEditEntity",
    component: WorkspaceEditEntity,
    props: true,
  },
  {
    path: "/o/:organizationId/w/:workspaceId",
    name: "workspace",
    component: Workspace,
    props: true,
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
