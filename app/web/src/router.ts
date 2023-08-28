import { createRouter, createWebHistory, RouteRecordRaw } from "vue-router";
import * as _ from "lodash-es";
import { nextTick } from "vue";
import { posthog } from "@/utils/posthog";
import { useAuthStore } from "./store/auth.store";
import { useRouterStore } from "./store/router.store";

// Cannot use inside the template directly.
const isDevMode = import.meta.env.DEV;
const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;

const routes: RouteRecordRaw[] = [
  {
    path: "/store-test",
    name: "store-test",
    meta: { public: true },
    component: () => import("@/pages/store_test/StoreTestPage.vue"),
  },
  {
    path: "/diagram",
    name: "diagram",
    meta: { public: true },
    component: () => import("@/components/GenericDiagram/DiagramDemoPage.vue"),
  },
  {
    path: "/",
    name: "home",
    component: () => import("@/pages/HomePage.vue"),
  },
  {
    path: "/w",
    name: "workspace-index",
    redirect: { name: "home" },
  },
  {
    name: "workspace-single",
    path: "/w/:workspacePk",
    component: () => import("@/pages/WorkspaceSinglePage.vue"),
    // TODO: will probably want a workspace "home" page at some point
    redirect(to) {
      return {
        name: "change-set-home",
        params: { ...to.params, changeSetId: "auto" },
      };
    },
    props: true,
    children: [
      {
        path: ":changeSetId",
        name: "change-set-home",
        // TODO: will probably want a change set "home" page at some point
        redirect(to) {
          return {
            name: "workspace-compose",
            params: to.params,
          };
        },
      },
      {
        path: ":changeSetId/c",
        name: "workspace-compose",
        component: () =>
          import("@/components/Workspace/WorkspaceModelAndView.vue"),
      },
      {
        path: ":changeSetId/l",
        name: "workspace-lab",
        component: () =>
          import("@/components/Workspace/WorkspaceCustomizeIndex.vue"),
        redirect(to) {
          return {
            name: "workspace-lab-assets",
            params: to.params,
          };
        },
        children: [
          {
            path: "a/:assetId?/:funcId?",
            name: "workspace-lab-assets",
            component: () =>
              import("@/components/Workspace/WorkspaceCustomizeAssets.vue"),
          },
          {
            path: "f/:funcId?",
            name: "workspace-lab-functions",
            component: () =>
              import("@/components/Workspace/WorkspaceCustomizeFunctions.vue"),
          },
          {
            path: "m/:moduleSlug?",
            name: "workspace-lab-packages",
            component: () =>
              import("@/components/Workspace/WorkspaceCustomizePackages.vue"),
          },
          {
            path: "s/:secretKind?",
            name: "workspace-lab-secrets",
            component: () =>
              import("@/components/Workspace/WorkspaceCustomizeSecrets.vue"),
          },
        ],
      },
      {
        path: "v",
        name: "workspace-view",
        component: () =>
          import("@/components/Workspace/WorkspaceModelAndView.vue"),
      },
      {
        path: "r",
        name: "workspace-fix",
        component: () => import("@/components/Workspace/WorkspaceApply.vue"),
      },
      ...(isDevMode
        ? [
            {
              path: "dev",
              name: "workspace-dev-dashboard",
              component: () =>
                import("@/components/Workspace/WorkspaceDevDashboard.vue"),
            },
          ]
        : []),
    ],
  },
  // Auth
  {
    path: "/auth-connect",
    name: "auth-connect",
    meta: { public: true },
    component: () => import("@/pages/auth/AuthConnectPage.vue"),
  },
  {
    path: "/login",
    name: "login",
    meta: { public: true },
    beforeEnter: () => {
      window.location.href = `${AUTH_PORTAL_URL}/login`;
    },
    component: () => import("@/pages/auth/AuthConnectPage.vue"),
  },
  {
    path: "/logout",
    name: "logout",
    beforeEnter: () => {
      const authStore = useAuthStore();
      authStore.localLogout();

      window.location.href = `${AUTH_PORTAL_URL}/logout`;
    },
    component: () => import("@/pages/auth/LogoutPage.vue"), // just need something here for TS, but guard always redirects
  },

  {
    path: "/oops",
    name: "oops",
    meta: { public: true },
    component: () => import("@/pages/OopsPage.vue"),
  },

  // 404
  {
    path: "/:catchAll(.*)",
    meta: { public: true },
    component: () => import("@/pages/NotFound.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach((to, _from) => {
  // check if meta info for route (or parent) requires auth or not
  // NOTE - this will not support a public parent with private children
  if (!to.matched.some((route) => route.meta.public)) {
    // check in auth store and redirect to login page if not logged in
    const authStore = useAuthStore();

    if (!authStore.userIsLoggedIn) {
      return {
        name: "login",
        ...(to.fullPath !== "/" && { query: { redirect: to.fullPath } }),
      };
    }
  }
  return true;
});

// set current route in the pinia store
// which is useful so we can set the currently selected workspace/change set from it
router.beforeResolve((to) => {
  const routerStore = useRouterStore();
  routerStore.currentRoute = to;
});

router.afterEach((to) => {
  nextTick(() => {
    posthog.capture("$pageview", {
      $current_url: to.fullPath,
    });
  });
});

export default router;
