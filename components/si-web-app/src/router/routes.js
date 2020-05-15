import SignInPage from "@/pages/SignInPage";
import NotFoundPage from "@/pages/NotFoundPage.vue";

const routes = [
  {
    path: "*",
    component: NotFoundPage,
  },
  {
    path: "/signin",
    name: "signin",
    component: SignInPage,
  }

];

export default routes;