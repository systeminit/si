// import Bottle from "bottlejs";
import { bottleSetup } from "./di";
import { config } from "./config";

import { createApp } from "vue";
// import VueRx from "vue-rx";
import App from "@/App.vue";
import router from "./router";

import "@/assets/main.css";
import "@/assets/tailwind.css";
import { SignupService } from "@/api/sdf/service/signup";

// import "@/plugins/vue-js-modal";

bottleSetup(config);
// let bottle = Bottle.pop("default");

// const RootComponent = {
//   // router: bottle.container.Router,
//   router: router,
//   render: h => h(App)
// }

const app = createApp(App);

// Expose our internal services to Cypress, so we can use them
// directly in tests.
// @ts-ignore
if (window.Cypress) {
  // @ts-ignore
  window.SignupService = SignupService;
}

// app.use(VueRx); // syntax not working

app.use(router);

app.mount("#app");
