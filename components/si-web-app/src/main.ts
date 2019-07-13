import Vue from "vue";
import "./plugins/vuetify";
import App from "./App.vue";
import router from "./router";
import AuthPlugin from "@/plugins/auth";
import AuthService from "@/auth/authService";

Vue.config.productionTip = false;

Vue.use(AuthPlugin);

declare module "vue/types/vue" {
  interface Vue {
    $auth: typeof AuthService;
  }
}

new Vue({
  router,
  render: h => h(App),
}).$mount("#app");
