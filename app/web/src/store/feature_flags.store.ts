import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { posthog } from "@/utils/posthog";

type FeatureFlag = UserFlag | WorkspaceFlag;
type UserFlag = keyof typeof USER_FLAG_MAPPING;
type WorkspaceFlag = keyof typeof WORKSPACE_FLAG_MAPPING;

// translation from store key to posthog feature flag name
const USER_FLAG_MAPPING = {
  // STORE_FLAG_NAME: "posthogFlagName",
  MODULES_TAB: "modules_tab",
  ADMIN_PANEL_ACCESS: "si_admin_panel_access",
  DIAGRAM_OPTIMIZATION_2: "diagram-optimization-2",
  AUTOCONNECT: "autoconnect-component-input-sockets",
  PRIVATE_SCOPED_MODULES: "private-scoped-modules",
  DIAGRAM_DRAG_LAYER: "diagram-drag-layer",
  SOCKET_VALUE: "socket-value",
  FLOATING_CONNECTION_MENU: "floating-connection-menu",
  SIMPLE_SOCKET_UI: "simple-socket-ui",
  SQLITE_TOOLS: "sqlite-tools",
  PROPS_TO_PROPS_CONNECTIONS: "props-to-props-connections",
  ENABLE_NEW_EXPERIENCE: "enable-new-experience",
  REVIEW_PAGE: "review-page",
  DEFAULT_SUBS: "default-subs",
  INITIALIZER_ONBOARD: "initializer-onboard",
  INITIALIZER_ONBOARD_FORCE_AGENT: "initializer-onboard-force-agent",
} as const;
const WORKSPACE_FLAG_MAPPING = {
  FRONTEND_ARCH_VIEWS: "workspace-frontend-arch-views",
  BIFROST_ACTIONS: "workspace-bifrost-actions",
  NEW_HOTNESS: "workspace-new-hotness",
} as const;

// List of all feature flags
const FEATURE_FLAGS = Object.keys({
  ...USER_FLAG_MAPPING,
  ...WORKSPACE_FLAG_MAPPING,
}) as FeatureFlag[];

export function useFeatureFlagsStore() {
  // const route = useRoute();
  // const workspacePk = route?.params?.workspacePk as WorkspacePk | undefined;
  // once the old UI is deleted, replace this with the commented out code above
  const workspacePk = window.location.pathname.split("/")[2];

  return addStoreHooks(
    undefined,
    undefined,
    defineStore("feature-flags", {
      // all flags default to undefined, but we put entries in the feature flags anyway
      state: () =>
        Object.fromEntries(
          FEATURE_FLAGS.map((flag) => [flag, undefined]),
        ) as Record<FeatureFlag, boolean | undefined>,
      getters: {
        allFeatureFlags: (state) =>
          FEATURE_FLAGS.map((name) => ({ name, value: state[name] })),
      },
      actions: {
        /**
         * Sets flags
         *
         * NOTE: This is deliberately not async, so that all flags are set at the same time and
         * there is no UI "flicker" if (for example) posthog has a flag as false but then we
         * set an override.
         *
         * DO NOT set feature flags anywhere else but here.
         *
         * @param featureFlags - Set of general feature flags from posthog
         * @param workspaceFlags - Set of workspace-specific feature flags from posthog
         */
        setFlags(featureFlags: Set<string>, workspaceFlags: Set<string>) {
          // Set the flags!
          for (const [flag, phFlag] of Object.entries(USER_FLAG_MAPPING)) {
            this[flag] = featureFlags.has(phFlag);
          }
          for (const [flag, phFlag] of Object.entries(WORKSPACE_FLAG_MAPPING)) {
            this[flag] = workspaceFlags.has(phFlag);
          }

          // You can override feature flags while working on a feature by setting them to true/false here
          // for example:
          // this.FEATURE_FLAG_NAME ??= false;
          // }
        },
        /**
         * Fetches workspace-specific feature flags
         *
         * If the response is not ok, it return an empty array.
         * @returns
         */
        async fetchWorkspaceFlags(): Promise<string[]> {
          if (!workspacePk) return [];
          try {
            const resp = await fetch(
              `${import.meta.env.VITE_POSTHOG_API_HOST}/decide/?v=3`,
              {
                method: "POST",
                body: JSON.stringify({
                  api_key: import.meta.env.VITE_POSTHOG_PUBLIC_KEY,
                  distinct_id: workspacePk,
                }),
              },
            );
            if (!resp.ok) {
              // TODO probably should just throw here
              // eslint-disable-next-line no-console
              console.error(
                `Error retrieving workspace-specific flags: ${resp}`,
              );
              return [];
            }
            const json = await resp.json();
            return Object.keys(json.featureFlags);
          } catch (err) {
            reportError(err);
            return [];
          }
        },
      },
      async onActivated() {
        // Grab workspace-specific flags once, and listen for feature flag changes from posthog
        const workspaceFlags = this.fetchWorkspaceFlags();
        // TODO remove feature flag listener on deactivate
        posthog.onFeatureFlags(async (flags) => {
          try {
            this.setFlags(new Set(flags), new Set(await workspaceFlags));
          } catch (e) {
            // eslint-disable-next-line no-console
            console.error("Error setting feature flags", e);
            throw e;
          }
        });
      },
    }),
  )();
}
