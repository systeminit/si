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
  ON_DEMAND_ASSETS: "on_demand_assets",
  AI_GENERATOR: "ai-generator",
  SLACK_WEBHOOK: "slack_webhook",
  TEMPLATE_MGMT_FUNC_GENERATION: "template-mgmt-func-generation",
};

const WORKSPACE_FLAG_MAPPING = {
  WORKSPACE_FINE_GRAINED_ACCESS_CONTROL:
    "workspace-fine-grained-access-control",
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
      state: () =>
        _.mapValues(
          { ...FLAG_MAPPING, ...WORKSPACE_FLAG_MAPPING },
          () => false,
        ),
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
        const result = await resp.json();
        Object.entries(WORKSPACE_FLAG_MAPPING).forEach(
          ([storeFlagKey, phFlag]) => {
            this[storeFlagKey] = result.featureFlags[phFlag] ?? false;
          },
        );

        // You can override feature flags while working on a feature by setting them to true/false here
        // for example:
        // this.MANAGEMENT_EDGES = false;
      },
    }),
  )();
}
