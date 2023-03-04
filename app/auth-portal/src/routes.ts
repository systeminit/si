import LoginPage from "./pages/LoginPage.vue";
import SignupPage from "./pages/SignupPage.vue";
import NotFoundPage from "./pages/NotFoundPage.vue";
import DashboardPage from "./pages/DashboardPage.vue";

// we export the routes instead of an actual router
// because ViteSSG handles the router in order to handle SSG
export default [
  { path: "/", name: "home", redirect: { name: "login" } },
  { path: "/signup", name: "signup", component: SignupPage },
  { path: "/login", name: "login", component: LoginPage },
  { path: "/dashboard", name: "dashboard", component: DashboardPage },
  { path: "/:catchAll(.*)", name: "404", component: NotFoundPage },
];
