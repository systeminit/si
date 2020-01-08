import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import VueGtag from "vue-gtag";
import { createProvider } from "./vue-apollo";
import { auth } from "./auth";
import vuetify from "./plugins/vuetify";

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

new Vue({
  router,
  apolloProvider,
  vuetify,
  render: h => h(App),
}).$mount("#app");
