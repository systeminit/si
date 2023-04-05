import * as _ from "lodash-es";
import { RouterOptions } from "vite-ssg";
import { Router } from "vue-router";
import LoginPage from "./pages/LoginPage.vue";
import LogoutPage from "./pages/LogoutPage.vue";
import NotFoundPage from "./pages/NotFoundPage.vue";
import DashboardPage from "./pages/DashboardPage.vue";
import LegalAgreementPage from "./pages/legal/LegalAgreementPage.vue";
import PrintLegalPage from "./pages/legal/PrintLegalPage.vue";
import ProfilePage from "./pages/ProfilePage.vue";
import TutorialPage from "./pages/tutorial/TutorialPage.vue";

// normally we'd initialze a router directly, but instead we pass the options to ViteSSG
export const routerOptions: RouterOptions = {
  routes: [
    { path: "/", name: "home", redirect: { name: "login" } },
    { path: "/login", name: "login", component: LoginPage },
    { path: "/logout", name: "logout", component: LogoutPage },
    {
      // public legal page, optionally can jump to specific doc
      path: "/legal/:docSlug?",
      name: "legal",
      component: LegalAgreementPage,
    },
    {
      // same legal page, but with "accept" checkbox/button
      path: "/review-legal",
      name: "review-legal",
      component: LegalAgreementPage,
    },
    {
      // special page showing single specific doc in format ready to print
      path: "/print-legal/:docVersion/:docSlug",
      name: "print-legal",
      component: PrintLegalPage,
    },
    { path: "/profile", name: "profile", component: ProfilePage },
    { path: "/tutorial", name: "tutorial", component: TutorialPage },
    { path: "/dashboard", name: "dashboard", component: DashboardPage },

    // auth api redirects to this route - gives us some flexibility with what to do with user
    // also used as a sort catch-all for "go to whatever is next"
    // App.vue has logic to kick user back to TOS/profile if necessary
    // this will let us toggle it back to the dashboard when the velvet-rope tutorial goes away
    {
      path: "/login-success",
      name: "login-success",
      redirect: { name: "tutorial" },
    },
    { path: "/:catchAll(.*)", name: "404", component: NotFoundPage },
  ],
};

export function initRouterGuards(router: Router) {
  router.beforeEach((from, to) => {
    if (!to.name || !_.isString(to.name)) return;
    if (["login", "logout", "404"].includes(to.name)) return;
    // TODO: might want to do something here...?
  });
}
