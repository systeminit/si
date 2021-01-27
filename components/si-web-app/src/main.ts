import Bottle from "bottlejs";
import { bottleSetup } from "@/di";
import Vue from "vue";
import App from "@/App.vue";
import router from "@/router";

import VueGtag from "vue-gtag";

// @ts-ignore
import { storeData } from "@/store";

import "@/assets/main.css";
import "@/assets/tailwind.css";

import "@/plugins/vue-js-modal";

bottleSetup(storeData);
let bottle = Bottle.pop("default");

Vue.config.productionTip = false;

if (process.env.NODE_ENV == "production") {
  Vue.use(VueGtag, {
    config: { id: "UA-151349900-2" },
  });
} else {
  Vue.use(VueGtag, {
    config: { id: "UA-151349900-2" },
    disableScriptLoad: true,
  });
}

new Vue({
  router,
  store: bottle.container.Store,
  render: h => h(App),
}).$mount("#app");
