import * as _ from "lodash-es";
import { RouterOptions } from "vite-ssg";
import { Router } from "vue-router";
import { nextTick } from "vue";
import posthog from "posthog-js";
import storage from "local-storage-fallback";
import WorkspaceDetailsPage from "@/pages/WorkspaceDetailsPage.vue";
import QuarantineNoticePage from "@/pages/QuarantineNoticePage.vue";
import SuspendedNoticePage from "@/pages/SuspendedNoticePage.vue";
import LoginPage from "./pages/LoginPage.vue";
import LogoutPage from "./pages/LogoutPage.vue";
import LogoutSuccessPage from "./pages/LogoutSuccessPage.vue";
import NotFoundPage from "./pages/NotFoundPage.vue";
import WorkspacesPage from "./pages/WorkspacesPage.vue";
import BillingPage from "./pages/BillingPage.vue";
import ProfilePage from "./pages/ProfilePage.vue";
import WorkspaceGoPage from "./pages/WorkspaceGoPage.vue";
import WorkspaceAdmin from "./pages/WorkspaceAdmin.vue";
import WorkspaceAuthTokensPage from "./pages/WorkspaceAuthTokensPage.vue";
import RumReport from "./pages/RumReport.vue";

// normally we'd initialize a router directly, but instead we pass the options to ViteSSG
export const routerOptions: RouterOptions = {
  routes: [
    { path: "/", name: "home", redirect: { name: "login" } },
    { path: "/login", name: "login", component: LoginPage },
    { path: "/signup", name: "signup", component: LoginPage },
    { path: "/logout", name: "logout", component: LogoutPage },
    {
      path: "/logout-success",
      name: "logout-success",
      component: LogoutSuccessPage,
    },
    {
      // public legal page, optionally can jump to specific doc
      path: "/legal/:docSlug?",
      name: "legal",
      component: () => import("./pages/legal/LegalAgreementPage.vue"),
    },
    {
      // same legal page, but with "accept" checkbox/button
      path: "/review-legal",
      name: "review-legal",
      component: () => import("./pages/legal/LegalAgreementPage.vue"),
    },
    {
      // special page showing single specific doc in format ready to print
      path: "/print-legal/:docVersion/:docSlug",
      name: "print-legal",
      component: () => import("./pages/legal/PrintLegalPage.vue"),
    },
    { path: "/profile", name: "profile", component: ProfilePage },
    {
      path: "/dashboard",
      name: "dashboard",
      redirect() {
        return { name: "workspaces" };
      },
    },
    {
      path: "/tutorial",
      name: "tutorial",
      redirect() {
        return { name: "workspaces" };
      },
    },
    {
      path: "/download",
      name: "download",
      redirect() {
        return { name: "workspaces" };
      },
    },
    { path: "/workspaces", name: "workspaces", component: WorkspacesPage },
    { path: "/billing", name: "billing", component: BillingPage },
    {
      path: "/default_workspace",
      name: "default-workspace",
      redirect() {
        return { name: "workspaces" };
      },
    },
    {
      path: "/workspace/:workspaceId",
      name: "workspace-settings",
      component: WorkspaceDetailsPage,
      props: true,
    },
    {
      path: "/workspace/:workspaceId/tokens",
      name: "workspace-api-tokens",
      component: WorkspaceAuthTokensPage,
      props: true,
    },
    {
      path: "/workspace/:workspaceId/go",
      name: "workspace-go",
      component: WorkspaceGoPage,
      props: true,
    },
    {
      path: "/workspace-admin",
      name: "workspace-admin",
      component: WorkspaceAdmin,
    },
    {
      path: "/rum-report",
      name: "rum-report",
      component: RumReport,
    },

    // auth api redirects to this route - gives us some flexibility with what to do with user
    // also used as a sort catch-all for "go to whatever is next"
    // App.vue has logic to kick user back to TOS/profile if necessary
    {
      path: "/login-success",
      name: "login-success",
      redirect() {
        if (import.meta.env.SSR) return { name: "workspaces" };

        // see App.vue for logic saving this redirect location
        const savedPath = storage.getItem("SI-LOGIN-REDIRECT");
        storage.removeItem("SI-LOGIN-REDIRECT");
        return savedPath || { name: "workspaces" };
      },
    },
    {
      path: "/quarantine-notice",
      name: "quarantine-notice",
      component: QuarantineNoticePage,
    },
    {
      path: "/suspension-notice",
      name: "suspension-notice",
      component: SuspendedNoticePage,
    },
    { path: "/:catchAll(.*)", name: "404", component: NotFoundPage },
  ],
};

export function initRouterGuards(router: Router) {
  router.beforeEach((from, to) => {
    if (!to.name || !_.isString(to.name)) return;
    if (["login", "signup", "logout", "404"].includes(to.name)) return;
    // TODO: might want to do something here...?
  });

  router.afterEach((to) => {
    nextTick(() => {
      posthog.capture("$pageview", { $current_url: to.fullPath });
      // eslint-disable-next-line no-console
    }).catch((e) => console.log("Failed to capture posthog pageview", e));
  });
}
