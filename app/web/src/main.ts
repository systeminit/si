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
//
// FIXME(paulo): Please don't judge me, I'm turning a bad hack (@ts-ignore)
// that causes a linter error into a less worse hack that causes a linter warning,
// so we can properly fix when we decide tackle all 'any' related technical debt
if ((window as any).Cypress) {
  (window as any).SignupService = SignupService;
  (window as any).SessionService = SessionService;
  (window as any).ChangeSetService = ChangeSetService;
}

app.use(router);

app.mount("#app");
