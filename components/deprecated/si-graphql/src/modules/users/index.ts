import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getUserById } from "./queries";
import { createUser } from "./mutations";

export const Users = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getUserById,
    },
    Mutation: {
      createUser,
    },
  },
});
