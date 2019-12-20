import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import { getWorkspaceById, getWorkspaces } from "./queries";
import { createWorkspace, deleteWorkspace } from "./mutations";
import { creator, members, createdWorkspaces, workspaces } from "./fields";

export const Workspaces = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getWorkspaceById,
      getWorkspaces,
    },
    Mutation: {
      createWorkspace,
      deleteWorkspace,
    },
    Workspace: {
      creator,
      members,
    },
    User: {
      createdWorkspaces,
      workspaces,
    },
  },
});
