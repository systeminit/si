import { bottleSetup } from "./di";
import { config } from "./config";

import { createApp } from "vue";
import App from "@/App.vue";
import router from "./router";

import "@/assets/main.css";
import "@/assets/tailwind.css";
import { SignupService } from "@/service/signup";
import { SessionService } from "@/service/session";
import { ChangeSetService } from "@/service/change_set";

bottleSetup(config);

const app = createApp(App);

// Expose our internal services to Cypress, so we can use them
// directly in tests.
// @ts-ignore
if (window.Cypress) {
  // @ts-ignore
  window.SignupService = SignupService;
  // @ts-ignore
  window.SessionService = SessionService;
  // @ts-ignore
  window.ChangeSetService = ChangeSetService;
}

app.use(router);

app.mount("#app");
