// import NotFoundPage from "@/pages/NotFound.vue";
// import Home from "@/pages/Home.vue";
// import Application from "@/templates/Application.vue";
// import ApplicationDetails from "@/templates/ApplicationDetails.vue";
// import Secret from "@/templates/Secret.vue";
import Authenticate from "@/pages/Authenticate.vue";
// import Login from "@/templates/Login.vue";
import Signup from "@/templates/Signup.vue";
import { RouteRecordRaw } from "vue-router";

// @ts-ignore
const routes: RouteRecordRaw[] = [
  // {
  //   path: "*",
  //   component: NotFoundPage,
  // },
  // {
  //   path: "/",
  //   name: "home",
  //   component: Home,
  //   children: [
  //     {
  //       path: "o/:organizationId/w/:workspaceId",
  //       props: true,
  //       name: "workspace",
  //       redirect: { name: "application" },
  //     },
  //     {
  //       path: "o/:organizationId/w/:workspaceId/a",
  //       props: true,
  //       name: "application",
  //       component: Application,
  //     },
  //     {
  //       path: "o/:organizationId/w/:workspaceId/a/:applicationId",
  //       props: true,
  //       name: "applicationDetails",
  //       component: ApplicationDetails,
  //     },
  //     {
  //       path: "o/:organizationId/w/:workspaceId/s",
  //       props: true,
  //       name: "secret",
  //       component: Secret,
  //     },
  //   ],
  // },
  {
    path: "/authenticate",
    name: "authenticate",
    component: Authenticate,
    // redirect: { name: "login" },
    redirect: { name: "signup" },
    children: [
      // {
      //   path: "login",
      //   name: "login",
      //   component: Login,
      // },
      {
        path: "signup",
        name: "signup",
        component: Signup,
      },
    ],
  },
];

export default routes;
