import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import posthog from "posthog-js";

export const useFeatureFlagsStore = () => {
  return addStoreHooks(
    undefined,
    undefined,
    defineStore("feature-flags", {
      state: () => ({
        ADMIN_PAGE: false,
        ON_DEMAND_ASSETS: false,
        CHANGE_USER_ROLE: false,
        AUTOMATION_API: false,
      }),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          this.ADMIN_PAGE = flags.includes("auth_portal_admin_page");
          this.ON_DEMAND_ASSETS = flags.includes("on_demand_assets");
          this.CHANGE_USER_ROLE = flags.includes("change_user_role");
          this.AUTOMATION_API = flags.includes("automation_api");
        });
      },
    }),
  )();
};
