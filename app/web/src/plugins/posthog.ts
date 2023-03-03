// ./plugins/posthog.js
import posthog from "posthog-js";
import { App, Plugin, nextTick } from "vue";
import router from "../router";

export const PosthogPlugin: Plugin = {
  install(app: App) {
    posthog.init("phc_SoQak5PP054RdTumd69bOz7JhM0ekkxxTXEQsbn3Zg9", {
      api_host: "https://app.posthog.com",
    });
    app.provide("posthog", posthog);
    router.afterEach((to) => {
      nextTick(() => {
        posthog.capture("$pageview", {
          $current_url: to.fullPath,
        });
      });
    });
  },
};
