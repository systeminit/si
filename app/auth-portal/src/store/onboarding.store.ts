import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { addStoreHooks } from "@si/vue-lib/pinia";
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
          stepsCompleted: {
            // pre-tutorial / other
            frienda: false,
            // temporarily let users through automatically if email is @systeminit
            // will be removed when feature flags are all set up
            github_access: authStore.user?.email?.endsWith("@systeminit.com"),
            // tutoral steps ------------
            intro: true, // we just always consider the intro step complete
            dev_setup: false,
            model: false,
            apply: false,
            cleanup: false,
            model_survey: false,
            customize: false,
            customize_survey: false,
            next_steps: false,
          },

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
          _.each(
            {
              frienda: "vro-frienda_accepted",
              github_access: "vro-github_access_granted",
              // tutorial steps (NOTE - intro doesn't have a flag as it starts complete)
              dev_setup: "vro-dev_setup_completed",
              model: "vro-model_completed",
              apply: "vro-apply_completed",
              cleanup: "vro-cleanup_completed",
              model_survey: "vro-model_survey_completed",
              customize: "vro-customize_completed",
              customize_survey: "vro-customize_survey_completed",
              // next_steps has no step to complete... so we mark it complete when everything else is done
              next_steps: "vro-customize_survey_completed",
            } as Record<keyof typeof this.stepsCompleted, string>,
            (featureToggleName, stepSlug) => {
              if (posthog.isFeatureEnabled(featureToggleName)) {
                this.stepsCompleted[
                  stepSlug as keyof typeof this.stepsCompleted
                ] = true;
              }
            },
          );
        },

        async checkDevEnvOnline() {
          try {
            const _req = await Axios.get("http://localhost:8080/up.txt");
            this.devFrontendOnline = true;
          } catch (err) {
            this.devFrontendOnline = false;
          }

          try {
            // hitting SDF via the front-end api proxy...
            // probably want to change this, but will need to adjust cors settings to do so
            const _req = await Axios.get("http://localhost:8080/api");
            // const _req = await Axios.get("http://localhost:5156/api");
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

        acknowledgeFrienda() {
          // track acceptance locally
          this.stepsCompleted.frienda = true;
          // fire off tracking event which will toggle the feature flag
          tracker.trackEvent("frienda_accepted");
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
