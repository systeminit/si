import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
import { watch } from "vue";
import { posthog } from "posthog-js";
import { useAuthStore } from "./auth.store";

const REFRESH_FEATURE_FLAG_FREQUENCY = 10000;

export const useOnboardingStore = () => {
  return addStoreHooks(
    defineStore("onboarding", {
      state: () => {
        const authStore = useAuthStore();

        return {
          // temporarily let users through automatically if email is @systeminit
          // will be removed when feature flags are all set up
          githubAccessGranted:
            authStore.user?.email?.endsWith("@systeminit.com") &&
            authStore.user.emailVerified,

          // in the backend we save an object of { [stepSlug]: [timestampCompletedAt] }
          // here we just remap to a boolean
          stepsCompleted: _.mapValues(
            authStore.user?.onboardingDetails?.vroStepsCompletedAt,
            (timestamp, _stepName) => !!timestamp,
          ),
        };
      },
      getters: {},
      actions: {
        handleNewFeatureFlags() {
          // check if github access gate has been lifted
          // it is a feature flag but should tell us they have been added to the repo
          if (posthog.isFeatureEnabled("vro-team1")) {
            this.githubAccessGranted = true;
          }
        },
      },
      onActivated() {
        const authStore = useAuthStore();

        let refreshFeatureFlagInterval: ReturnType<typeof setInterval>;
        posthog.onFeatureFlags(this.handleNewFeatureFlags);

        const stopWatchLoggedIn = watch(
          () => authStore.userIsLoggedIn,
          () => {
            if (authStore.userIsLoggedIn) {
              refreshFeatureFlagInterval = setInterval(() => {
                posthog.reloadFeatureFlags();
              }, REFRESH_FEATURE_FLAG_FREQUENCY);
            } else {
              clearInterval(refreshFeatureFlagInterval);
            }
          },
          { immediate: true },
        );

        return () => {
          stopWatchLoggedIn();
          clearInterval(refreshFeatureFlagInterval);
        };
      },
    }),
  )();
};
