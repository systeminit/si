import { ViteSSG } from "vite-ssg";

import "./style.css";
import App from "./App.vue";
import routes from "./routes";
import store from "./store";

// createApp(App).mount('#app')
// const app = createSSRApp(App)
// app.mount('#app')

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
