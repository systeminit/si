// import Bottle from "bottlejs";
import { bottleSetup } from "./di";
import { config } from "./config";

import { createApp } from "vue";
// import VueRx from "vue-rx";
import App from "@/App.vue";
import router from "./router";

import VueGtag from "vue-gtag";

import "@/assets/main.css";
import "@/assets/tailwind.css";

// import "@/plugins/vue-js-modal";

bottleSetup(config);
// let bottle = Bottle.pop("default");

// const RootComponent = {
//   // router: bottle.container.Router,
//   router: router,
//   render: h => h(App)
// }

const app = createApp(App);

// app.use(VueRx); // syntax not working

app.use(router);

if (process.env.NODE_ENV == "production") {
  app.use(VueGtag, {
    config: { id: "UA-151349900-2" },
  });
} else {
  app.use(VueGtag, {
    config: { id: "UA-151349900-2" },
    disableScriptLoad: true,
  });
}

app.mount("#app");
