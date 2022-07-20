import { config } from "@/config";
import {
  createWebHistory,
  createRouter,
  RouteRecordRaw,
  RouteLocationNormalized,
  NavigationGuardNext,
} from "vue-router";
import { SessionService } from "@/service/session";
import Home from "@/pages/Home.vue";
import NotFoundPage from "@/pages/NotFound.vue";
import Authenticate from "@/pages/Authenticate.vue";
import Login from "@/templates/Login.vue";
import Signup from "@/templates/Signup.vue";
import Schema from "@/templates/Schema.vue";
import SchemaList from "@/organisims/Schema/SchemaList.vue";
import SchemaNew from "@/organisims/Schema/SchemaNew.vue";
import SchemaView from "@/organisims/Schema/SchemaView.vue";
import Application from "@/templates/Application.vue";
import ApplicationList from "@/organisims/Application/ApplicationList.vue";
import ApplicationView from "@/organisims/Application/ApplicationView.vue";
import Editor from "@/organisims/Editor.vue";
import _ from "lodash";
import SchematicViewer from "@/organisims/SchematicViewer.vue";
import NewHome from "@/pages/NewHome.vue";
import WorkspaceSingle from "@/templates/WorkspaceSingle.vue";
import WorkspaceMultiple from "@/templates/WorkspaceMultiple.vue";
import WorkspaceView from "@/organisims/Workspace/WorkspaceView.vue";
import WorkspaceRuntime from "@/organisims/Workspace/WorkspaceRuntime.vue";
import WorkspaceCompose from "@/organisims/Workspace/WorkspaceCompose.vue";
import WorkspaceLab from "@/organisims/Workspace/WorkspaceLab.vue";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "home",
    component: Home,
    children: [
      {
        path: "application",
        props: true,
        name: "application",
        component: Application,
        redirect: { name: "application-list" },
        children: [
          {
            name: "application-list",
            path: "list",
            component: ApplicationList,
            props: { modal: true },
          },
          {
            name: "application-new",
            path: "new",
            component: Editor,
          },
          {
            name: "application-view",
            path: ":applicationId",
            props: (route) => {
              let applicationId;
              if (_.isArray(route.params.applicationId)) {
                applicationId = Number.parseInt(route.params.applicationId[0]);
              } else {
                applicationId = Number.parseInt(route.params.applicationId);
              }
              return {
                applicationId,
              };
            },
            component: ApplicationView,
          },
        ],
      },
      {
        path: "schema",
        props: true,
        name: "schema",
        component: Schema,
        redirect: { name: "schema-list" },
        children: [
          {
            name: "schema-list",
            path: "list",
            component: SchemaList,
            props: { modal: true },
          },
          {
            name: "schema-new",
            path: "new",
            component: SchemaNew,
          },
          {
            name: "schema-view",
            path: ":schemaId",
            props: (route) => {
              let schemaId;
              if (_.isArray(route.params.schemaId)) {
                schemaId = Number.parseInt(route.params.schemaId[0]);
              } else {
                schemaId = Number.parseInt(route.params.schemaId);
              }
              return {
                schemaId,
              };
            },
            component: SchemaView,
          },
        ],
      },
      {
        path: "schematic",
        props: true,
        name: "schematic",
        component: SchematicViewer,
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
    path: "/new",
    name: "new",
    component: NewHome,
    children: [
      {
        path: "w",
        name: "workspace-multiple",
        component: WorkspaceMultiple,
        redirect: { name: "new" },
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
