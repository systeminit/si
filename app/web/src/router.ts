import { config } from "@/config";
import {
  createRouter,
  createWebHistory,
  NavigationGuardNext,
  RouteLocationNormalized,
  RouteRecordRaw,
} from "vue-router";
import { SessionService } from "@/service/session";
import NotFoundPage from "@/pages/NotFound.vue";
import Authenticate from "@/pages/Authenticate.vue";
import Login from "@/templates/Login.vue";
import Signup from "@/templates/Signup.vue";
import _ from "lodash";
import Home from "@/pages/Home.vue";
import WorkspaceSingle from "@/templates/WorkspaceSingle.vue";
import WorkspaceMultiple from "@/templates/WorkspaceMultiple.vue";
import WorkspaceView from "@/organisms/Workspace/WorkspaceView.vue";
import WorkspaceRuntime from "@/organisms/Workspace/WorkspaceRuntime.vue";
import WorkspaceCompose from "@/organisms/Workspace/WorkspaceCompose.vue";
import WorkspaceLab from "@/organisms/Workspace/WorkspaceLab.vue";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "home",
    component: Home,
    children: [
      {
        path: "w",
        name: "workspace-multiple",
        component: WorkspaceMultiple,
        redirect: { name: "home" },
        children: [
          {
            name: "workspace-single",
            path: ":workspaceId",
            component: WorkspaceSingle,
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
                component: WorkspaceCompose,
              },
              {
                path: "l",
                name: "workspace-lab",
                component: WorkspaceLab,
              },
              {
                path: "v",
                name: "workspace-view",
                component: WorkspaceView,
              },
              {
                path: "r",
                name: "workspace-runtime",
                component: WorkspaceRuntime,
              },
            ],
          },
        ],
      },
    ],
  },
  {
    path: "/authenticate",
    name: "authenticate",
    component: Authenticate,
    // redirect: { name: "login" },
    redirect: { name: "signup" },
    children: [
      {
        path: "login",
        name: "login",
        component: Login,
      },
      {
        path: "signup",
        name: "signup",
        component: Signup,
      },
    ],
  },
  {
    path: "/404",
    name: "notFound",
    component: NotFoundPage,
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
  if (to.path == "/authenticate/signup" || to.path == "/authenticate/login") {
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
