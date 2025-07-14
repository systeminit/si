import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { posthog } from "@/utils/posthog";
import { useWorkspacesStore } from "./workspaces.store";

// translation from store key to posthog feature flag name
const FLAG_MAPPING = {
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
};

const WORKSPACE_FLAG_MAPPING = {
  FRONTEND_ARCH_VIEWS: "workspace-frontend-arch-views",
  BIFROST_ACTIONS: "workspace-bifrost-actions",
  NEW_HOTNESS: "workspace-new-hotness",
};

const ALL_FLAG_MAPPING: Record<FeatureFlags, string> = {
  ...FLAG_MAPPING,
  ...WORKSPACE_FLAG_MAPPING,
};

type KeysOfUnion<T> = T extends T ? keyof T : never;
type FeatureFlags = KeysOfUnion<
  typeof FLAG_MAPPING | typeof WORKSPACE_FLAG_MAPPING
>;
const PH_TO_STORE_FLAG_LOOKUP = _.invert(FLAG_MAPPING) as Record<
  string,
  FeatureFlags
>;

export function useFeatureFlagsStore() {
  const workspacesStore = useWorkspacesStore();
  const workspacePk = workspacesStore.urlSelectedWorkspaceId;

  return addStoreHooks(
    undefined,
    undefined,
    defineStore("feature-flags", {
      // all flags default to false
      state: () => _.mapValues({ ...ALL_FLAG_MAPPING }, () => false),
      getters: {
        allFeatureFlags(state) {
          const flags = [] as Array<{ name: string; value: boolean }>;
          for (const key of Object.keys(ALL_FLAG_MAPPING)) {
            flags.push({ name: key, value: state[key as FeatureFlags] });
          }
          return flags;
        },
      },
      actions: {
        setDependentFlags() {
          if (this.ENABLE_NEW_EXPERIENCE) {
            this.FLOATING_CONNECTION_MENU = true;
            this.SIMPLE_SOCKET_UI = true;
            this.PROPS_TO_PROPS_CONNECTIONS = true;
          }
        },
      },
      async onActivated() {
        posthog.onFeatureFlags((phFlags) => {
          // reset local flags from posthog data
          _.each(phFlags, (phFlag) => {
            const storeFlagKey = PH_TO_STORE_FLAG_LOOKUP[phFlag];
            if (storeFlagKey) {
              this[storeFlagKey] = true;
            }
          });
        });

        // NOTE: this will return all the OTHER flags too... so only look for workspace specific ones
        const resp = await fetch(
          `${import.meta.env.VITE_POSTHOG_API_HOST}/decide/?v=3`,
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              api_key: import.meta.env.VITE_POSTHOG_PUBLIC_KEY,
              distinct_id: workspacePk,
            }),
          },
        );
        if (resp.ok) {
          const result = await resp.json();
          Object.entries(WORKSPACE_FLAG_MAPPING).forEach(
            ([storeFlagKey, phFlag]) => {
              this[storeFlagKey] = result.featureFlags[phFlag] ?? false;
            },
          );
        }

        // You can override feature flags while working on a feature by setting them to true/false here
        // for example:
        // this.FEATURE_FLAG_NAME = false;

        // turning this on for local development
        if (import.meta.env.VITE_SI_ENV === "local")
          this.ENABLE_NEW_EXPERIENCE = true;

        // After processing override flags, set dependent flags.
        this.setDependentFlags();
      },
    }),
  )();
}
