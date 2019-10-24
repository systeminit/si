import { GraphQLModule } from "@graphql-modules/core";

import typeDefs from "./schema.graphql";
import {
  getAllIntegrations,
  getIntegrationInstances,
  getIntegrationInstanceById,
} from "./queries";
import {
  createIntegrationInstance,
  deleteIntegrationInstance,
  enableIntegrationInstanceOnWorkspace,
  disableIntegrationInstanceOnWorkspace,
} from "./mutations";
import {
  integration,
  user,
  workspaces,
  integrationInstances,
  workspaceIntegrationInstances,
} from "./fields";

export const Integrations = new GraphQLModule({
  typeDefs,
  resolvers: {
    Query: {
      getAllIntegrations,
      getIntegrationInstances,
      getIntegrationInstanceById,
    },
    Mutation: {
      createIntegrationInstance,
      deleteIntegrationInstance,
      enableIntegrationInstanceOnWorkspace,
      disableIntegrationInstanceOnWorkspace,
    },
    IntegrationInstance: {
      integration,
      user,
      workspaces,
    },
    User: {
      integrationInstances,
    },
    Workspace: {
      integrationInstances: workspaceIntegrationInstances,
    },
  },
});
