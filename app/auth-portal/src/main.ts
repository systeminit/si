import { ViteSSG } from "vite-ssg";

import "@si/vue-lib/tailwind/main.css";
import "@si/vue-lib/tailwind/tailwind.css";

import App from "./App.vue";
import routes from "./routes";
import store from "./store";

export const createApp = ViteSSG(
  // the root component
  App,
  // vue-router options
  {
    routes,
  },
  // // function to have custom setups
  ({
    app,
    // router, routes, isClient, initialState
  }) => {
    // install plugins etc.
    app.use(store);
  },
);
