// ./plugins/posthog.js
import posthog from "posthog-js";
import { App, Plugin, nextTick } from "vue";
import router from "../router";

export const PosthogPlugin: Plugin = {
  install(_app: App) {
    posthog.init(import.meta.env.VITE_POSTHOG_PUBLIC_DEV_KEY, {
      api_host: import.meta.env.VITE_POSTHOG_API_HOST,
    });
    router.afterEach((to) => {
      nextTick(() => {
        posthog.capture("$pageview", {
          $current_url: to.fullPath,
        });
      });
    });
  },
};

export function usePosthog() {
  return posthog;
}
