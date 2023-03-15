import { RouteRecordRaw } from "vue-router";
import LoginPage from "./pages/LoginPage.vue";
import LogoutPage from "./pages/LogoutPage.vue";
import NotFoundPage from "./pages/NotFoundPage.vue";
import DashboardPage from "./pages/DashboardPage.vue";
import ReviewTosPage from "./pages/ReviewTosPage.vue";

// we export the routes instead of an actual router
// because ViteSSG handles the router in order to handle SSG
const routes: RouteRecordRaw[] = [
  { path: "/", name: "home", redirect: { name: "login" } },
  { path: "/login", name: "login", component: LoginPage },
  { path: "/logout", name: "logout", component: LogoutPage },
  { path: "/review-tos", name: "review-tos", component: ReviewTosPage },
  { path: "/dashboard", name: "dashboard", component: DashboardPage },
  { path: "/:catchAll(.*)", name: "404", component: NotFoundPage },
];

export default routes;
