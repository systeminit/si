import fetch from "node-fetch";
import { ApolloClient } from "apollo-client";
import { ApolloLink } from "apollo-link";
import { InMemoryCache } from "apollo-cache-inmemory";
import { HttpLink } from "apollo-link-http";

const cache = new InMemoryCache();
const headerLink = new ApolloLink((operation, forward) => {
  const apolloToken = localStorage.getItem("apollo-token");
  if (apolloToken) {
    operation.setContext({
      headers: {
        authorization: `Bearer ${apolloToken}`,
      },
    });
  }
  return forward(operation);
});
const link = headerLink.concat(
  new HttpLink({
    uri: "http://localhost:4000/",
    // @ts-ignore
    fetch,
  }),
);

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
