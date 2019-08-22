import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getServerComponents } from "./queries";

export const Servers = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getServerComponents,
    },
  },
});

