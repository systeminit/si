import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import { posthog } from "@/utils/posthog";

export function useFeatureFlagsStore() {
  return addStoreHooks(
    defineStore("feature-flags", {
      state: () => ({
        SINGLE_MODEL_SCREEN: null as null | boolean,
      }),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          this.SINGLE_MODEL_SCREEN = flags.includes(
            "one_screen_to_rule_them_all",
          );
        });
      },
    }),
  )();
}
