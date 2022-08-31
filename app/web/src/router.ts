import {
  createRouter,
  createWebHistory,
  NavigationGuardNext,
  RouteLocationNormalized,
  RouteRecordRaw,
} from "vue-router";
import _ from "lodash";
import { config } from "@/config";
import { SessionService } from "@/service/session";

const routes: RouteRecordRaw[] = [
  {
    path: "/diagram",
    name: "diagram",
    component: () => import("@/organisms/GenericDiagram/DiagramDemoPage.vue"),
  },
  {
    path: "/",
    name: "home",
    redirect: { name: "workspace-index" },
  },
  {
    path: "/w",
    name: "workspace-index",
    component: () => import("@/templates/WorkspaceIndex.vue"),
  },
  {
    name: "workspace-single",
    path: "/w/:workspaceId",
    component: () => import("@/templates/WorkspaceSingle.vue"),
    redirect: { name: "workspace-compose" },
    props: (route) => {
      let workspaceId;
      if (_.isArray(route.params.workspaceId)) {
        workspaceId = Number.parseInt(route.params.workspaceId[0]);
      } else {
        workspaceId = Number.parseInt(route.params.workspaceId);
      }
      return {
        workspaceId,
      };
    },
    children: [
      {
        path: "c",
        name: "workspace-compose",
        component: () => import("@/organisms/Workspace/WorkspaceCompose.vue"),
      },
      {
        path: "l/:funcId?",
        name: "workspace-lab",
        props: true,
        component: () => import("@/organisms/Workspace/WorkspaceLab.vue"),
      },
      {
        path: "v",
        name: "workspace-view",
        component: () => import("@/organisms/Workspace/WorkspaceCompose.vue"),
      },
      {
        path: "r",
        name: "workspace-runtime",
        component: () => import("@/organisms/Workspace/WorkspaceRuntime.vue"),
      },
    ],
  },
  {
    path: "/authenticate",
    name: "authenticate",
    component: () => import("@/pages/Authenticate.vue"),
    // redirect: { name: "login" },
    redirect: { name: "signup" },
    children: [
      {
        path: "login",
        name: "login",
        component: () => import("@/templates/Login.vue"),
      },
      {
        path: "signup",
        name: "signup",
        component: () => import("@/templates/Signup.vue"),
      },
    ],
  },
  {
    path: "/404",
    name: "notFound",
    component: () => import("@/pages/NotFound.vue"),
  },
  {
    path: "/:catchAll(.*)",
    redirect: "/404",
  },
];

export const routeCheck = async (
  to: RouteLocationNormalized,
  _from: RouteLocationNormalized,
  next: NavigationGuardNext,
) => {
  if (
    to.path === "/authenticate/signup" ||
    to.path === "/authenticate/login" ||
    to.path === "/diagram"
  ) {
    return next();
  }

  const authenticated = await SessionService.isAuthenticated();
  if (authenticated === false || authenticated.error) {
    if (authenticated.error) {
      console.log("Error checking authentication", authenticated);
    }
    return next("/authenticate/login");
  } else {
    return next();
  }
};

const router = createRouter({
  history: createWebHistory(config.routerBase),
  routes,
});

router.beforeEach(
  async (
    to: RouteLocationNormalized,
    from: RouteLocationNormalized,
    next: NavigationGuardNext,
  ) => {
    await routeCheck(to, from, next);
  },
);

export default router;
