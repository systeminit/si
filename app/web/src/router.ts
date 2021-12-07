import { config } from "@/config";
import { createWebHistory, createRouter, RouteRecordRaw } from "vue-router";
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
import Editor from "@/organisims/Editor.vue";
import _ from "lodash";

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
            component: Editor,
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

export const routeCheck = async (to: any, _from: any, next: any) => {
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

router.beforeEach(async (to, from, next) => {
  await routeCheck(to, from, next);
});

export default router;
