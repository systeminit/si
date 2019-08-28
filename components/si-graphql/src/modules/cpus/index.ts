import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getCpuComponents } from "./queries";

export const Cpus = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getCpuComponents,
    },
  },
});
