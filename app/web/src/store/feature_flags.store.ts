import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import { posthog } from "@/utils/posthog";

const FLAGS: { [key: string]: { key: string; default: boolean } } = {
  one_screen_to_rule_them_all: {
    key: "SINGLE_MODEL_SCREEN",
    default: false,
  },
  modules_tab: {
    key: "MODULES_TAB",
    default: false,
  },
};

const flagsToState = () =>
  Object.values(FLAGS).reduce((state, flag) => {
    state[flag.key] = flag.default;
    return state;
  }, {} as { [key: string]: boolean });

export function useFeatureFlagsStore() {
  return addStoreHooks(
    defineStore("feature-flags", {
      state: () => flagsToState(),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          for (const flag of flags) {
            const flagKey = FLAGS[flag]?.key;
            if (flagKey) {
              this[flagKey] = true;
            }
          }
        });
      },
    }),
  )();
}
