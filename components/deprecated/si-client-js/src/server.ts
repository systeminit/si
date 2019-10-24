import { SearchPrimitive } from "searchjs";
import gql from "graphql-tag";

import { apolloClient } from "@/apollo";
import { ServerComponent, ServerEntity } from "@/generated/graphql";

export interface ServerCreate {
  constraints?: SearchPrimitive, 
  args: {
    name: string,
    description?: string,
    operatingSystem?: SearchPrimitive,
    sshKey?: SearchPrimitive,
  },
}

async function create(req: ServerCreate): Promise<ServerEntity> {
  const input = {
    constraints: JSON.stringify(req.constraints),
    args: req.args,
    workspace: "1640dd40-f388-42e2-ab0a-7fafd36f8173",
  };
  console.log("I got died here tho");
  let result = await apolloClient.mutate({
    mutation: gql`
      mutation createServer($input: CreateServerInput) {
        createServer(
        input: $input
        ) {
          port {
            id
            name
            serviceName
            protocol
            description
            number
            component {
              id
              name
            }
          }
        }
      }`,
    variables: {input},
  });
  return result.data.createServer.port;
};

async function findComponent(args: SearchPrimitive): Promise<ServerComponent[]> {
  let result = await apolloClient.query({ 
    query: gql`query findServerComponents($search: String!, $workspace: String) {
  findServerComponents(where: { search: $search, workspace: $workspace }) {
      id
      name
      description
      rawDataJson
      nodeType
      supportedActions
      serviceName
      protocol
      number
      integration {
        id
        name
        description
      }
    }
  }`,
  variables: {
    search: JSON.stringify(args)
  }});
  return result.data.findServerComponents;
}

export const Server = {
  create,
  findComponent,
}

