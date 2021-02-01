import NotFoundPage from "@/pages/NotFoundPage.vue";
import HomePage from "@/pages/HomePage.vue";
import AuthenticatePage from "@/pages/AuthenticatePage.vue";
import LoginView from "@/pages/AuthenticatePage/LoginView.vue";
import SignupView from "@/pages/AuthenticatePage/SignupView.vue";
import { RouteConfig } from "vue-router";

const routes: RouteConfig[] = [
  {
    path: "*",
    component: NotFoundPage,
  },
  {
    path: "/",
    name: "home",
    component: HomePage,
  },
  {
    path: "/authenticate",
    name: "authenticate",
    component: AuthenticatePage,
    redirect: { name: "login" },
    children: [
      {
        path: "login",
        name: "login",
        component: LoginView,
      },
      {
        path: "signup",
        name: "signup",
        component: SignupView,
      },
    ],
  },
];

export default routes;
