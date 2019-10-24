import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getComponents, findComponents } from "./queries";

export const Components = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getComponents,
      findComponents,
    },
  },
});
