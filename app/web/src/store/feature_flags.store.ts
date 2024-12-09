import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { posthog } from "@/utils/posthog";

// translation from store key to posthog feature flag name
const FLAG_MAPPING = {
  // STORE_FLAG_NAME: "posthogFlagName",
  MODULES_TAB: "modules_tab",
  FIRST_TIME_TUTORIAL_MODAL: "first_time_tutorial_modal",
  DEV_SLICE_REBASING: "dev-slice-rebasing",
  ADMIN_PANEL_ACCESS: "si_admin_panel_access",
  ON_DEMAND_ASSETS: "on_demand_assets",
  MANAGEMENT_FUNCTIONS: "management-functions",
  MANAGEMENT_EDGES: "management-edges",
  AUDIT_PAGE: "audit-page",
  AI_GENERATOR: "ai-generator",
  REBAC: "rebac",
  OUTLINER_VIEWS: "diagram-outline-show-views",
  SLACK_WEBHOOK: "slack_webhook",
};

type FeatureFlags = keyof typeof FLAG_MAPPING;
const PH_TO_STORE_FLAG_LOOKUP = _.invert(FLAG_MAPPING) as Record<
  string,
  FeatureFlags
>;

export function useFeatureFlagsStore() {
  return addStoreHooks(
    undefined,
    undefined,
    defineStore("feature-flags", {
      // all flags default to false
      state: () => _.mapValues(FLAG_MAPPING, () => false),
      onActivated() {
        posthog.onFeatureFlags((phFlags) => {
          // reset local flags from posthog data
          _.each(phFlags, (phFlag) => {
            const storeFlagKey = PH_TO_STORE_FLAG_LOOKUP[phFlag];
            if (storeFlagKey) {
              this[storeFlagKey as FeatureFlags] = true;
            }
          });
        });
        // You can override feature flags while working on a feature by setting them to true/false here
        // for example:
        // this.MANAGEMENT_EDGES = false;
      },
    }),
  )();
}
