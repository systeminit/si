import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getOperatingSystemComponents } from "./queries";

export const OperatingSystems = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getOperatingSystemComponents,
    },
  },
});
