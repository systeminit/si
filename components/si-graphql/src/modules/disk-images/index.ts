import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getDiskImageComponents } from "./queries";

export const DiskImages = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getDiskImageComponents,
    },
  },
});
