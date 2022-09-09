import { createApp } from "vue";
import FloatingVue from "floating-vue";
import VueKonva from "vue-konva";
import { create } from "rxjs-spy";
import { createHead } from "@vueuse/head";

import "@/assets/style/main.css";
import "@/assets/style/tailwind.css";
import { ChangeSetService } from "@/service/change_set";
import { SessionService } from "@/service/session";
import { SignupService } from "@/service/signup";
import App from "@/App.vue";
import { bottleSetup } from "./di";
import router from "./router";

// @ts-ignore
const _spy = create();

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

bottleSetup();

const app = createApp(App);

const head = createHead();
app.use(head);

app.use(router);

// we attach to the #app-layout div (in AppLayout.vue) to stay within an overflow hidden div and not mess with page scrollbars
app.use(FloatingVue, { container: "#app-layout" });

// unfortunately, vue-konva only works as a global plugin, so we must register it here
// TODO: fork the lib and set it up so we can import individual components
app.use(VueKonva);

app.mount("#app");
