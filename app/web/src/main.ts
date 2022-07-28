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
import FloatingVue from "floating-vue";
import VueKonva from "vue-konva";
import { create } from "rxjs-spy";
// @ts-ignore
const _spy = create();

bottleSetup(config);

const app = createApp(App);
app.use(FloatingVue);

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

// unfortunately, vue-konva only works as a global plugin, so we must register it here
// TODO: fork the lib and set it up so we can import individual components
app.use(VueKonva);

app.mount("#app");
