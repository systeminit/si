import NotFoundPage from "@/pages/NotFoundPage";
import Home from "@/pages/HomePage.vue";
import SignInPage from "@/pages/SignInPage";
import WorkspacePage from "@/pages/WorkspacePage";
import SystemDetails from "@/components/views/system/SystemDetails.vue";

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
        props: true
      }
    ]
  },
];

export default routes;