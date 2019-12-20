import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import { createProvider } from "./vue-apollo";
import { auth } from "./auth";
import vuetify from "./plugins/vuetify";

Vue.config.productionTip = false;

let apolloProvider = createProvider();
auth.setApollo(apolloProvider);

new Vue({
  router,
  apolloProvider,
  vuetify,
  render: h => h(App),
}).$mount("#app");
