import { ViteSSG } from "vite-ssg";
import FloatingVue from "floating-vue";

import "@si/vue-lib/tailwind/main.css";
import "@si/vue-lib/tailwind/tailwind.css";

import "@/assets/extra-style.less";

import "./lib/posthog";
import { FLOATING_VUE_THEMES } from "@si/vue-lib/design-system";
import App from "./App.vue";
import store from "./store";
import { initRouterGuards, routerOptions } from "./router";

export const createApp = ViteSSG(
  // the root component
  App,
  // vue-router options - routes defined there
  routerOptions,
  // // function to have custom setups
  ({
    app,
    router,
    isClient,
    // routes, initialState
  }) => {
    // install plugins etc.
    app.use(store);
    app.use(FloatingVue, {
      themes: FLOATING_VUE_THEMES,
    });

    if (isClient) initRouterGuards(router);
  },
);
