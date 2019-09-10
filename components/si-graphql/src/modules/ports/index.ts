import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getPortComponents, findPortComponents } from "./queries";
import { createPort } from "./mutations";

export const Ports = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getPortComponents,
      findPortComponents,
    },
    Mutation: {
      createPort,
    },
  },
});
