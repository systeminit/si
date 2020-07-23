import "@/utils/telemetry";
import Vue from "vue";
import App from "@/App.vue";
import router from "@/router";

import VueGtag from "vue-gtag";

// @ts-ignore
import store from "@/store";

import "@/assets/main.css";
import "@/assets/tailwind.css";

import "@/plugins/vue-js-modal";

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
  store,
  render: h => h(App),
}).$mount("#app");
