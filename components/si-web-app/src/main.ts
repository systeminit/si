import "@/utils/telemetry";
import Vue from "vue";
import App from "@/App.vue";
import router from "@/router";

import VueGtag from "vue-gtag";
import vSelect from 'vue-select'
import { createProvider } from "@/plugins/vue-apollo";
import { auth } from "@/utils/auth";

// @ts-ignore
import store from '@/store'

import "@/assets/main.css";
import "@/assets/tailwind.css";
import "@/plugins/vue-tailwind.js";

import "@/plugins/vue-js-modal.js";
import "@/plugins/vue-simple-context-menu.js";


Vue.config.productionTip = false;

let apolloProvider = createProvider();
auth.setApollo(apolloProvider);

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

Vue.component('v-select', vSelect)

new Vue({
  router,
  apolloProvider,
  store,
  render: h => h(App),
}).$mount("#app");
