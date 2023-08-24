import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import { posthog } from "@/utils/posthog";

type FlagStoreKey =
  | "SINGLE_MODEL_SCREEN"
  | "MODULES_TAB"
  | "CONTRIBUTE_BUTTON"
  | "SECRETS";

// The key to this object is the flag name on PostHog, the "storeKey" property is the
// key used when accessing it from the Pinia feature flag store
const FLAGS: { [key: string]: { storeKey: FlagStoreKey; default: boolean } } = {
  one_screen_to_rule_them_all: {
    storeKey: "SINGLE_MODEL_SCREEN",
    default: false,
  },
  modules_tab: {
    storeKey: "MODULES_TAB",
    default: false,
  },
  contribute_button: {
    storeKey: "CONTRIBUTE_BUTTON",
    default: false,
  },
  secrets: {
    storeKey: "SECRETS",
    default: false,
  },
};

const flagsToState = () =>
  Object.values(FLAGS).reduce((state, flag) => {
    state[flag.storeKey] = flag.default;
    return state;
  }, {} as { [key in FlagStoreKey]: boolean });

export function useFeatureFlagsStore() {
  return addStoreHooks(
    defineStore("feature-flags", {
      state: () => flagsToState(),
      onActivated() {
        posthog.onFeatureFlags((flags) => {
          for (const flag of flags) {
            const flagKey = FLAGS[flag]?.storeKey;
            if (flagKey) {
              this[flagKey] = true;
            }
          }
        });
      },
    }),
  )();
}
