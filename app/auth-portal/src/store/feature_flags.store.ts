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
        OSS_RELEASE: true, // todo: cleanup consumption of this flag
        CREATE_WORKSPACES: false,
        EDIT_WORKSPACES: false,
        DELETE_WORKSPACE: false,
        SIMPLIFIED_SIGNUP: false,
        ADMIN_PAGE: false,
        SAAS_RELEASE: false,
        ON_DEMAND_ASSETS: false,
        CHANGE_USER_ROLE: false,
      }),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          this.OSS_RELEASE = flags.includes("featureOssRelease");
          this.CREATE_WORKSPACES = flags.includes("create_workspaces");
          // If you can create workspaces, editing workspaces will also be enabled.
          this.DELETE_WORKSPACE = flags.includes("delete_workspace");
          this.SIMPLIFIED_SIGNUP = flags.includes("simplified_signup_flow");
          this.EDIT_WORKSPACES =
            flags.includes("edit_workspaces") || this.CREATE_WORKSPACES;
          this.ADMIN_PAGE = flags.includes("auth_portal_admin_page");
          this.SAAS_RELEASE = flags.includes("auth_portal_saas_release");
          this.ON_DEMAND_ASSETS = flags.includes("on_demand_assets");
          this.CHANGE_USER_ROLE = flags.includes("change_user_role");
        });
      },
    }),
  )();
};
