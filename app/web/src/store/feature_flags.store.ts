import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { posthog } from "@/utils/posthog";

// translation from store key to posthog feature flag name
const FLAG_MAPPING = {
  // STORE_FLAG_NAME: "posthogFlagName",
  MODULES_TAB: "modules_tab",
  SECRETS: "secrets",
  INVITE_USER: "invite_user",
  SECRETS_MANAGEMENT: "secrets_management",
  WORKSPACE_BACKUPS: "workspaceBackups",
  COLLABORATORS: "collaborators",
  MUTLIPLAYER_CHANGESET_APPLY: "multiplayer_changeset_apply_flow",
  ABANDON_CHANGESET: "abandon_changeset",
  CONNECTION_ANNOTATIONS: "socket_connection_annotations",
  COPY_PASTE: "copy_paste",
  DONT_BLOCK_ON_ACTIONS: "dont_block_on_actions",
  OVERRIDE_SCHEMA: "override_schema",
};

type FeatureFlags = keyof typeof FLAG_MAPPING;
const PH_TO_STORE_FLAG_LOOKUP = _.invert(FLAG_MAPPING) as Record<
  string,
  FeatureFlags
>;

export function useFeatureFlagsStore() {
  return addStoreHooks(
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
        // You can override feature flags while working on a feature by setting them to true here

        // Make sure to remove the override before committing your code!
      },
    }),
  )();
}
