import Vue from "vue";
import VueApollo from "vue-apollo";
import {
  createApolloClient,
  restartWebsockets,
} from "vue-cli-plugin-apollo/graphql-client";
import ApolloClient from "apollo-client";
import { setContext } from "apollo-link-context";
import { ApolloLink } from "apollo-link";
import { InMemoryCache } from "apollo-cache-inmemory";
import { SubscriptionClient } from "subscriptions-transport-ws";
import * as api from "@opentelemetry/api";
import { telemetry, tracer } from "@/utils/telemetry";
import { print as printGql } from "graphql/language/printer";

// Install the vue plugin
Vue.use(VueApollo);

// Name of the localStorage item
const AUTH_TOKEN = "apollo-token";

let httpEndpoint =
  process.env.VUE_APP_GRAPHQL_HTTP || "http://localhost:4000/graphql";
let wsEndpoint =
  process.env.VUE_APP_GRAPHQL_WS || "ws://localhost:4000/graphql";
if (process.env.NODE_ENV === "production") {
  httpEndpoint = "https://graphql.systeminit.com/graphql";
  wsEndpoint = "wss://graphql.systeminit.com/graphql";
}

const telemetryLink = setContext((request, prevContext) => {
  let spanName = `web.graphql.`;
  if (request.operationName) {
    spanName += request.operationName;
  } else {
    spanName += "anon";
  }
  const span = telemetry.activitySpan(`${spanName}`);
  span.setAttributes({
    "web.graphql.name": request.operationName || "anon",
    "web.graphql.operationName": request.operationName,
    "web.graphql.query": printGql(request.query),
    "web.graphql.variables": JSON.stringify(request.variables),
  });
  const headers = tracer.withSpan(span, () => {
    const headers: Record<string, unknown> = {};
    api.propagation.inject(headers, (headers, k, v) => {
      headers[k] = v;
    });
    return headers;
  });
  return {
    headers: { traceparent: headers["traceparent"], ...prevContext["headers"] },
    telemetrySpan: span,
  };
});

const afterwareLink = new ApolloLink((operation, forward) => {
  return forward(operation).map(response => {
    const context = operation.getContext();
    if (context.telemetrySpan) {
      if (response.errors) {
        context.telemetrySpan.setAttribute({ error: true });
        context.telemetrySpan.setAttribute({
          "web.graphql.errors": JSON.stringify(response.errors),
        });
      }
      context.telemetrySpan.end();
    }
    return response;
  });
});

const customLink = afterwareLink.concat(telemetryLink);

// Config
const defaultOptions = {
  // You can use `https` for secure connection (recommended in production)
  httpEndpoint,
  // You can use `wss` for secure connection (recommended in production)
  // Use `null` to disable subscriptions
  wsEndpoint: wsEndpoint,
  // LocalStorage token
  tokenName: AUTH_TOKEN,
  // Enable Automatic Query persisting with Apollo Engine
  persisting: false,
  // Use websockets for everything (no HTTP)
  // You need to pass a `wsEndpoint` for this to work
  websocketsOnly: false,
  // Is being rendered on the server?
  ssr: false,

  // Override default apollo link
  // note: don't override httpLink here, specify httpLink options in the
  // httpLinkOptions property of defaultOptions.
  link: customLink,

  // Override default cache
  // cache: myCache

  // Override the way the Authorization header is set
  // getAuth: (tokenName) => ...

  // Additional ApolloClient options
  // apollo: { ... }

  // Client local data (see apollo-link-state)
  // clientState: { resolvers: { ... }, defaults: { ... } }
};

export type ExtendedApolloClient = ApolloClient<InMemoryCache> & {
  wsClient: SubscriptionClient;
};

// Call this in the Vue app file
export function createProvider(options = {}) {
  // Create apollo client
  const { apolloClient, wsClient } = createApolloClient({
    ...defaultOptions,
    ...options,
  });
  const myApolloClient: ExtendedApolloClient = apolloClient as ExtendedApolloClient;
  myApolloClient.wsClient = wsClient;

  // Create vue apollo provider
  const apolloProvider = new VueApollo({
    defaultClient: myApolloClient,
    defaultOptions: {
      $query: {
        // fetchPolicy: 'cache-and-network',
      },
    },
    errorHandler(error) {
      console.log(
        "%cError",
        "background: red; color: white; padding: 2px 4px; border-radius: 3px; font-weight: bold;",
        error.message,
      );
    },
  });

  return apolloProvider;
}

// Manually call this when user log in
export async function onLogin(
  apolloClient: ExtendedApolloClient,
  token: string,
) {
  if (typeof localStorage !== "undefined" && token) {
    localStorage.setItem(AUTH_TOKEN, token);
  }
  if (apolloClient.wsClient) restartWebsockets(apolloClient.wsClient);
  try {
    await apolloClient.resetStore();
  } catch (e) {
    console.log("%cError on cache reset (login)", "color: orange;", e.message);
  }
}

// Manually call this when user log out
export async function onLogout(apolloClient: ExtendedApolloClient) {
  if (typeof localStorage !== "undefined") {
    localStorage.removeItem(AUTH_TOKEN);
  }
  if (apolloClient.wsClient) restartWebsockets(apolloClient.wsClient);
  try {
    await apolloClient.resetStore();
  } catch (e) {
    console.log("%cError on cache reset (logout)", "color: orange;", e.message);
  }
}
