import NotFoundPage from "@/pages/NotFound.vue";
import Home from "@/pages/Home.vue";
import Application from "@/templates/Application.vue";
import ApplicationDetails from "@/templates/ApplicationDetails.vue";
import Authenticate from "@/pages/Authenticate.vue";
import Login from "@/templates/Login.vue";
import Signup from "@/templates/Signup.vue";
import { RouteConfig } from "vue-router";

const routes: RouteConfig[] = [
  {
    path: "*",
    component: NotFoundPage,
  },
  {
    path: "/",
    name: "home",
    component: Home,
    children: [
      {
        path: "o/:organizationId/w/:workspaceId",
        props: true,
        name: "workspace",
        redirect: { name: "application" },
      },
      {
        path: "o/:organizationId/w/:workspaceId/a",
        props: true,
        name: "application",
        component: Application,
      },
      {
        path: "o/:organizationId/w/:workspaceId/a/:applicationId",
        props: true,
        name: "applicationDetails",
        component: ApplicationDetails,
      },
    ],
  },
  {
    path: "/authenticate",
    name: "authenticate",
    component: Authenticate,
    redirect: { name: "login" },
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
];

export default routes;
