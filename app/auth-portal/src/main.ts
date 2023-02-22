import { ViteSSG } from "vite-ssg";
import "./style.css";
import App from "./App.vue";

import LoginPage from "./pages/LoginPage.vue";
import SignupPage from "./pages/SignupPage.vue";
import NotFoundPage from "./pages/NotFoundPage.vue";

// createApp(App).mount('#app')
// const app = createSSRApp(App)
// app.mount('#app')

export const createApp = ViteSSG(
  // the root component
  App,
  // vue-router options
  {
    routes: [
      { name: "home", path: "/", redirect: { name: "login" } },
      { name: "signup", path: "/signup", component: SignupPage },
      { name: "login", path: "/login", component: LoginPage },
      { name: "404", path: "/:catchAll(.*)", component: NotFoundPage },
    ],
  },
  // // function to have custom setups
  // ({ app, router, routes, isClient, initialState }) => {
  //   // install plugins etc.
  // },
);
