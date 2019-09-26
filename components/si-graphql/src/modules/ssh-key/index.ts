import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import {
  getSshKeyComponents,
  findSshKeyComponents,
} from "@/modules/ssh-key/queries";
import { createSshKey } from "@/modules/ssh-key/mutations";

export const SshKey = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getSshKeyComponents,
      findSshKeyComponents,
    },
    Mutation: {
      createSshKey,
    },
  },
});
