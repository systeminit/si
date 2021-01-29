import NotFoundPage from "@/pages/NotFoundPage.vue";
import Home from "@/pages/HomePage.vue";
import SignInPage from "@/pages/SignInPage/index.vue";
import SignUpPage from "@/pages/SignUpPage.vue";
import LoadingPage from "@/pages/LoadingPage.vue";
import WorkspacePage from "@/pages/WorkspacePage/index.vue";
import SystemDetails from "@/components/views/system/SystemDetails.vue";
import ApplicationList from "@/components/views/application/ApplicationList.vue";
import SecretList from "@/components/views/secret/SecretList.vue";
import ClientList from "@/components/views/client/ClientList.vue";
import ApplicationDetails from "@/components/views/application/ApplicationDetails.vue";

const routes = [
  {
    path: "*",
    component: NotFoundPage,
  },
  {
    path: "/",
    name: "home",
    component: Home,
  },
  {
    path: "/signin",
    name: "signin",
    component: SignInPage,
  },
  {
    path: "/signup",
    name: "signup",
    component: SignUpPage,
  },

  {
    path: "/loading",
    name: "loading",
    component: LoadingPage,
  },
  {
    path: "/o/:organizationId/w/:workspaceId",
    name: "workspace",
    component: WorkspacePage,
    props: true,
    children: [
      {
        path: "s/:systemId",
        name: "system",
        component: SystemDetails,
        props: true,
      },
      {
        path: "a/:applicationId",
        name: "applicationDetails",
        component: ApplicationDetails,
        props: true,
      },
      {
        path: "a",
        name: "application",
        component: ApplicationList,
        props: true,
      },
      {
        path: "secrets",
        name: "secret",
        component: SecretList,
        props: true,
      },
      {
        path: "clients",
        name: "client",
        component: ClientList,
        props: true,
      },
      {
        path: "global",
        name: "global",
        component: SystemDetails,
        props: true,
      },
    ],
  },
];

export default routes;
