import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getComponents } from "./queries";

export const Components = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getComponents,
    },
  },
});
