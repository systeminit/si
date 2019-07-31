import Vue from "vue";
import { ApolloClient } from "apollo-client";
import { setContext } from "apollo-link-context";
import { createHttpLink } from "apollo-link-http";
import { InMemoryCache } from "apollo-cache-inmemory";
import VueApollo from "vue-apollo";

import vuetify from "./plugins/vuetify";
import App from "./App.vue";
import router from "./router";
import AuthPlugin from "@/plugins/auth";
import AuthService from "@/auth/authService";

Vue.config.productionTip = false;

// HTTP connection to the API
const httpLink = createHttpLink({
  // You should use an absolute URL here
  uri: "http://localhost:4000/graphql",
});

// Cache implementation
const cache = new InMemoryCache();

// If we have an authentication token in local storage, append it
// to all of our Apollo calls
const authLink = setContext((_, { headers }) => {
  // get the authentication token from local storage if it exists
  const token = localStorage.getItem("authIdToken");
  // return the headers to the context so httpLink can read them
  return {
    headers: {
      ...headers,
      authorization: token ? `Bearer ${token}` : "",
    },
  };
});

// Create the apollo client
const apolloClient = new ApolloClient({
  link: authLink.concat(httpLink),
  cache,
});

Vue.use(AuthPlugin);
Vue.use(VueApollo);

const apolloProvider = new VueApollo({
  defaultClient: apolloClient,
});

declare module "vue/types/vue" {
  interface Vue {
    $auth: typeof AuthService;
  }
}

new Vue({
  router,
  apolloProvider,
  vuetify,
  render: h => h(App),
}).$mount("#app");
