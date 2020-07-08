import fetch from "node-fetch";
import { ApolloClient } from "apollo-client";
import { ApolloLink } from "apollo-link";
import { InMemoryCache } from "apollo-cache-inmemory";
import { HttpLink } from "apollo-link-http";
import { registry } from "si-registry";

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

interface GraphqlQueryArgs extends QueryArgs {
  typeName: string;
  variables?: Record<string, any>;
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

