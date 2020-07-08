import { ApolloClient, ApolloQueryResult } from "apollo-client";
import { ApolloLink, FetchResult } from "apollo-link";
import { setContext } from "apollo-link-context";
import { onError } from "apollo-link-error";
import { InMemoryCache } from "apollo-cache-inmemory";
import { HttpLink } from "apollo-link-http";
import * as api from "@opentelemetry/api";
import { print as printGql } from "graphql/language/printer";

import { telemetry, tracer } from "@/utils/telemetry";
import { registry, QueryArgs } from "si-registry";

interface GraphqlQueryArgs extends QueryArgs {
  typeName: string;
  variables?: Record<string, any>;
}

// Name of the localStorage item
const AUTH_TOKEN = "apollo-token";

let httpEndpoint =
  process.env.VUE_APP_GRAPHQL_HTTP || "http://localhost:4000/graphql";
if (process.env.NODE_ENV === "production") {
  httpEndpoint = "https://graphql.systeminit.com/graphql";
}

const cache = new InMemoryCache({
  addTypename: false,
});

const httpLink = new HttpLink({
  uri: httpEndpoint,
});

const authLink = new ApolloLink((operation, forward) => {
  const apolloToken = localStorage.getItem(AUTH_TOKEN);
  if (apolloToken) {
    operation.setContext({
      headers: {
        authorization: `Bearer ${apolloToken}`,
      },
    });
  }
  return forward(operation);
});

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

const errorLink = onError(({ operation, graphQLErrors, networkError }) => {
  if (graphQLErrors)
    graphQLErrors.forEach(({ message, locations, path }) =>
      console.log(
        `[GraphQL error]: Operation: ${
          operation.operationName
        }, Message: ${message}, Location: ${locations?.join(
          ", ",
        )}, Path: ${path}, Variables: ${JSON.stringify(operation.variables)}`,
      ),
    );
  if (networkError) console.log(`[Network error]: ${networkError}`);
});

const link = ApolloLink.from([
  authLink,
  telemetryLink,
  afterwareLink,
  errorLink,
  httpLink,
]);

export const apollo = new ApolloClient({
  // Provide required constructor fields
  cache: cache,
  link: link,
  defaultOptions: {
    query: {
      fetchPolicy: "no-cache",
    },
    mutate: {
      fetchPolicy: "no-cache",
    },
  },
});

export async function onLogin(token: string) {
  if (typeof localStorage !== "undefined" && token) {
    localStorage.setItem(AUTH_TOKEN, token);
  }
  try {
    await apollo.resetStore();
  } catch (e) {
    console.log("%cError on cache reset (login)", "color: orange;", e.message);
  }
}

export async function onLogout() {
  if (typeof localStorage !== "undefined") {
    localStorage.removeItem(AUTH_TOKEN);
  }
  try {
    await apollo.resetStore();
  } catch (e) {
    console.log("%cError on cache reset (logout)", "color: orange;", e.message);
  }
}

export async function graphqlQuery(
  args: GraphqlQueryArgs,
): Promise<Record<string, any>> {
  const siObject = registry.get(args.typeName);
  const query = siObject.graphql.query(args);

  const rawResults = await apollo.query({
    query,
    variables: args.variables,
  });
  return siObject.graphql.extractResult({
    methodName: args.methodName,
    data: rawResults,
  });
}

export async function graphqlMutation(
  args: GraphqlQueryArgs,
): Promise<Record<string, any>> {
  const siObject = registry.get(args.typeName);
  const mutation = siObject.graphql.mutation(args);
  const rawResults = await apollo.mutate({
    mutation,
    variables: args.variables,
  });
  return siObject.graphql.extractResult({
    methodName: args.methodName,
    data: rawResults,
  });
}

type QueryListAllArgs = Omit<GraphqlQueryArgs, "methodName">;
export async function graphqlQueryListAll(
  args: QueryListAllArgs,
): Promise<Record<string, any>[]> {
  let remainingItems = true;
  let nextPageToken = "";
  let results: any[] = [];

  while (remainingItems) {
    let itemList;
    if (nextPageToken) {
      itemList = await graphqlQuery({
        methodName: "list",
        variables: {
          pageToken: nextPageToken,
        },
        ...args,
      });
    } else {
      itemList = await graphqlQuery({
        methodName: "list",
        variables: {
          pageSize: "100",
        },
        ...args,
      });
    }
    results = results.concat(itemList["items"]);
    nextPageToken = itemList["nextPageToken"];
    if (!nextPageToken) {
      remainingItems = false;
    }
  }
  return results;
}
