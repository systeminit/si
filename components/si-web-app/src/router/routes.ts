import NotFoundPage from "@/pages/NotFoundPage.vue";
import Home from "@/pages/HomePage.vue";
import SignInPage from "@/pages/SignInPage/index.vue";
import WorkspacePage from "@/pages/WorkspacePage/index.vue";
import SystemDetails from "@/components/views/system/SystemDetails.vue";
import ApplicationList from "@/components/views/application/ApplicationList.vue";
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
    path: "/o/:organizationId/w/:workspaceId",
    name: "workspace",
    component: WorkspacePage,
    props: true,
    children: [
      {
        path: "/o/:organizationId/w/:workspaceId/s/:systemId",
        name: "system",
        component: SystemDetails,
        props: true,
      },
      {
        path: "/o/:organizationId/w/:workspaceId/a",
        name: "application",
        component: ApplicationList,
        props: true,
      },
      {
        path: "/o/:organizationId/w/:workspaceId/a/:applicationId",
        name: "applicationDetails",
        component: ApplicationDetails,
        props: true,
      },
      {
        path: "/o/:organizationId/w/:workspaceId/global",
        name: "global",
        component: SystemDetails,
        props: true,
      },
    ],
  },
];

export default routes;
