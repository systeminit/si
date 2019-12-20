import fetch from "cross-fetch";
import { ApolloClient } from 'apollo-client';
import { InMemoryCache } from 'apollo-cache-inmemory';
import { setContext } from 'apollo-link-context';
import { HttpLink } from 'apollo-link-http';

import { env } from "@/environment";

// Instantiate required constructor fields
const cache = new InMemoryCache();
const httpLink = new HttpLink({
  uri: 'http://localhost:4000/graphql',
  fetch: fetch,
});
const authLink = setContext((_, { headers }) => {
  return {
    headers: {
      ...headers,
      authorization: env.apollo.authorization ? `Bearer ${env.apollo.authorization}` : "",
    }
  }
});

export const apolloClient = new ApolloClient({
  // Provide required constructor fields
  cache: cache,
  link: authLink.concat(httpLink),

  // Provide some optional constructor fields
  name: 'si-client-js',
  version: '0.0.1',
  queryDeduplication: false,
  defaultOptions: {
    watchQuery: {
      fetchPolicy: 'cache-and-network',
    },
  },
});
