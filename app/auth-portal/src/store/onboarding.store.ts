import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { watch } from "vue";
import Axios from "axios";
import { posthog } from "posthog-js";
import { tracker } from "@/lib/posthog";
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

          devBackendOnline: false,
          devFrontendOnline: false,
        };
      },
      getters: {
        devInstanceOnline: (state) =>
          state.devBackendOnline && state.devFrontendOnline,
      },
      actions: {
        handleNewFeatureFlags() {
          // check if github access gate has been lifted
          // it is a feature flag but should tell us they have been added to the repo
          if (posthog.isFeatureEnabled("vro-team1")) {
            this.githubAccessGranted = true;
          }
          // _.each(
          //   {
          //     // tutorial steps (NOTE - intro doesn't have a flag as it starts complete)
          //     dev_setup: "vro-dev_setup_completed",
          //     model: "vro-model_completed",
          //     apply: "vro-apply_completed",
          //     cleanup: "vro-cleanup_completed",
          //     model_survey: "vro-model_survey_completed",
          //     customize: "vro-customize_completed",
          //     customize_survey: "vro-customize_survey_completed",
          //     // next_steps has no step to complete... so we mark it complete when everything else is done
          //     next_steps: "vro-customize_survey_completed",
          //   } as Record<keyof typeof this.stepsCompleted, string>,
          //   (featureToggleName, stepSlug) => {
          //     if (posthog.isFeatureEnabled(featureToggleName)) {
          //       this.stepsCompleted[
          //         stepSlug as keyof typeof this.stepsCompleted
          //       ] = true;
          //     }
          //   },
          // );
        },

        async checkDevEnvOnline() {
          try {
            const _req = await Axios.get("http://localhost:8080/up.txt");
            this.devFrontendOnline = true;
          } catch (err) {
            this.devFrontendOnline = false;
          }

          try {
            const _req = await Axios.get("http://localhost:5156/api/");
            this.devBackendOnline = true;
          } catch (err) {
            this.devBackendOnline = false;
          }

          // this is first time user has dev setup online, track it
          if (
            !this.stepsCompleted.dev_setup &&
            this.devFrontendOnline &&
            this.devBackendOnline
          ) {
            this.stepsCompleted.dev_setup = true;
            tracker.trackEvent("dev_env_online");
            // will toggle posthog.isFeatureEnabled("vro_dev-setup-completed") &&
          }
        },
        async COMPLETE_TUTORIAL_STEP(step: string) {
          const authStore = useAuthStore();
          if (!authStore.user) throw new Error("Must be logged in");
          const userId = authStore.user?.id;

          /* eslint-disable @typescript-eslint/no-explicit-any */
          const stepAlreadyCompleted = (this.stepsCompleted as any)[step];
          if (!stepAlreadyCompleted) {
            tracker.trackEvent("vro_tutorial_step_completed", { step });
            (this.stepsCompleted as any)[step] = true;
          }

          return new ApiRequest({
            method: "post",
            url: `/users/${userId}/complete-tutorial-step`,
            params: { step },
            // we dont care about the response since we track it being completed locally
            // and only care about what the api returns when we refresh
          });
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

        const checkDevEnvInteval = setInterval(this.checkDevEnvOnline, 5000);
        // eslint-disable-next-line @typescript-eslint/no-floating-promises
        this.checkDevEnvOnline();

        return () => {
          stopWatchLoggedIn();
          clearInterval(refreshFeatureFlagInterval);
          clearInterval(checkDevEnvInteval);
        };
      },
    }),
  )();
};
