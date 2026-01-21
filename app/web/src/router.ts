import { createRouter, createWebHistory, RouteRecordRaw } from "vue-router";
import * as _ from "lodash-es";
import { nextTick } from "vue";
import { AxiosRequestConfig } from "axios";
import { posthog } from "@/utils/posthog";
import { push as pushBreadcrumb, query } from "@/newhotness/logic_composables/navigation_stack";
import { tokensByWorkspacePk } from "@/newhotness/logic_composables/tokens";
import { useAuthStore } from "./store/auth.store";
import { useRouterStore } from "./store/router.store";
import { isDevMode } from "./utils/debug";
import { sdfApiInstance as sdf } from "./store/apis.web";
import { WorkspaceMetadata } from "./api/sdf/dal/workspace";

// Cannot use inside the template directly.
const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;

const findChangeSetId = async (workspacePk: string | undefined) => {
  /**
   * PSA: this may throw a 401 if the tokens aren't yet stored from a new
   * successful login. The 401 is nothing to be alarmed about, it doesn't stop
   * the app from continuing to work, because we're not throwing an exception below
   * with the `validateStatus`
   */
  const token = tokensByWorkspacePk[(workspacePk as string) || ""];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const params: AxiosRequestConfig<any> = {
    method: "GET",
    url: `v2/workspaces/${workspacePk}/change-sets`,
    validateStatus: (_status) => true, // don't throw exception on 4/5xxx
  };
  if (token) {
    params.headers = {
      Authorization: `Bearer ${token}`,
    };
  }
  const resp = await sdf<WorkspaceMetadata>(params);
  return resp.data.defaultChangeSetId;
};

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "home",
    component: () => import("@/pages/HomePage.vue"),
  },
  {
    path: "/hashes",
    name: "hashes",
    meta: { public: true },
    component: () => import("@/pages/Hashes.vue"),
  },
  {
    path: "/w",
    name: "workspace-index",
    redirect: { name: "home" },
  },
  {
    name: "new-hotness-workspace",
    path: "/n/:workspacePk",
    beforeEnter: async (loc) => {
      const changeSetId = await findChangeSetId(loc.params.workspacePk as string | undefined);
      const newloc = `/n/${loc.params.workspacePk}/${changeSetId}/h`;
      return newloc;
    },
    component: () => import("@/newhotness/Workspace.vue"),
  },
  {
    name: "new-hotness-workspace-auto",
    path: "/n/:workspacePk/auto",
    beforeEnter: async (loc) => {
      const changeSetId = await findChangeSetId(loc.params.workspacePk as string | undefined);
      const newloc = `/n/${loc.params.workspacePk}/${changeSetId}/h`;
      return newloc;
    },
    component: () => import("@/newhotness/Workspace.vue"),
  },
  {
    name: "new-hotness-head",
    path: "/n/:workspacePk/head/h",
    beforeEnter: async (loc) => {
      const changeSetId = await findChangeSetId(loc.params.workspacePk as string | undefined);
      const newloc = `/n/${loc.params.workspacePk}/${changeSetId}/h`;
      return newloc;
    },
    component: () => import("@/newhotness/Workspace.vue"),
  },
  {
    name: "new-hotness",
    path: "/n/:workspacePk/:changeSetId/h",
    props: true,
    component: () => import("@/newhotness/Workspace.vue"),
    children: [
      // {
      //   name: "new-hotness-lobby",
      //   path: "lobby",
      //   component: () => import("@/newhotness/Workspace.vue"),
      // },
      {
        name: "new-hotness-view",
        path: ":viewId/v/edit",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
      {
        name: "new-hotness-secrets-list",
        path: "secrets",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
      {
        name: "new-hotness-component",
        path: ":componentId/c",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
      {
        name: "new-hotness-policy-home",
        path: "p",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
      {
        name: "new-hotness-policy",
        path: ":policyId/:policyName/p",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
      {
        name: "new-hotness-func-run",
        path: ":funcRunId/r",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
      {
        name: "new-hotness-action",
        path: ":actionId/a",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
      {
        name: "new-hotness-review",
        path: "r",
        props: true,
        component: () => import("@/newhotness/Workspace.vue"),
      },
    ],
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
        redirect: { name: "new-hotness" },
        children: [
          {
            path: ":viewId/v/",
            name: "workspace-compose-view",
            redirect: { name: "new-hotness" },
          },
        ],
      },
      {
        path: ":changeSetId/l",
        name: "workspace-lab",
        component: () => import("@/components/Workspace/WorkspaceCustomizeIndex.vue"),
        redirect(to) {
          return {
            name: "workspace-lab-assets",
            params: to.params,
            query: to.query,
          };
        },
        children: [
          {
            path: "a/",
            name: "workspace-lab-assets",
            component: () => import("@/components/Workspace/WorkspaceCustomizeAssets.vue"),
          },
          {
            path: "n/",
            name: "workspace-lab-newassets",
            component: () => import("@/components/Workspace/WorkspaceCustomizeAssets.vue"),
          },
          {
            path: "m/:moduleSlug?",
            name: "workspace-lab-packages",
            component: () => import("@/components/Workspace/WorkspaceCustomizeModules.vue"),
          },
        ],
      },
      {
        path: ":changeSetId/a",
        name: "workspace-audit",
        component: () => import("@/components/Workspace/WorkspaceAuditLog.vue"),
      },
      {
        path: "admin",
        name: "workspace-admin-dashboard",
        component: () => import("@/components/Workspace/WorkspaceAdminDashboard.vue"),
      },
      ...(isDevMode
        ? [
            {
              path: "dev",
              name: "workspace-dev-dashboard",
              component: () => import("@/components/Workspace/WorkspaceDevDashboard.vue"),
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
    path: "/refresh-auth",
    name: "refresh-auth",
    meta: { public: true },
    component: () => import("@/pages/auth/RefreshAuthPage.vue"),
  },
  {
    path: "/login",
    name: "login",
    meta: { public: true },
    beforeEnter: (route) => {
      const query = route.query;
      const workspaceId = query.workspaceId;
      const queryString = query.redirect ? `redirect=${query.redirect}` : "";

      if (workspaceId) {
        window.location.href = `${AUTH_PORTAL_URL}/workspace/${workspaceId}/go?${queryString}`;
      } else {
        window.location.href = `${AUTH_PORTAL_URL}/login?${queryString}`;
      }
    },
    component: () => import("@/pages/auth/AuthConnectPage.vue"),
  },
  {
    path: "/logout",
    name: "logout",
    beforeEnter: async () => {
      const authStore = useAuthStore();
      await authStore.logout();
    },
    component: () => import("@/pages/auth/LogoutPage.vue"), // just need something here for TS, but guard always redirects
  },

  {
    path: "/oops",
    name: "oops",
    meta: { public: true },
    component: () => import("@/pages/OopsPage.vue"),
  },

  ...(isDevMode
    ? [
        // svg debug page, see all icons and svgs in the system in one place
        {
          path: "/w/:workspacePk/svg",
          name: "svg",
          meta: { public: true },
          component: () => import("@/pages/DebugPage.vue"),
        },
      ]
    : []),

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
      const workspaceId = to.fullPath.match(/^\/(n|w)\/(?<workspaceId>\w*)\//)?.groups?.workspaceId;

      return {
        name: "login",
        ...(to.fullPath !== "/" && {
          query: {
            redirect: to.fullPath,
            workspaceId,
          },
        }),
      };
    }
  }
  return true;
});

// set current route in the pinia store
// which is useful so we can set the currently selected workspace/change set from it
router.beforeResolve((to) => {
  const routerStore = useRouterStore();
  // setting the route we intend to navigate to
  routerStore.currentRoute = to;
});

router.afterEach((to) => {
  let name = "unknown";
  try {
    // NOTE: the `matched` array is least specific to most specific, so the one you want is the last one
    const _name = to.matched[to.matched.length - 1]?.name?.toString();
    if (_name) name = _name;
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error("can't find name in matched route", e);
  }
  try {
    pushBreadcrumb(to.path, name, { ...to.params }, { ...to.query } as query);
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error("can't push breadcrumb", e);
  }

  nextTick(() => {
    posthog.capture("$pageview", {
      $current_url: to.fullPath,
    });
  });
});

export default router;
