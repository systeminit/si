import { SearchPrimitive } from "searchjs";
import gql from "graphql-tag";

import { apolloClient } from "@/apollo";
import { PortComponent, PortEntity } from "@/generated/graphql";

interface PortCreate {
  constraints?: SearchPrimitive, 
  args: {
    name: string,
    description?: string,
    serviceName?: string,
    protocol?: string,
    number?: number,
  },
}

async function create(req: PortCreate): Promise<PortEntity> {
  const input = {
    constraints: JSON.stringify(req.constraints),
    args: req.args,
    workspace: "1640dd40-f388-42e2-ab0a-7fafd36f8173",
  };
  console.log("I got died here tho");
  let result = await apolloClient.mutate({
    mutation: gql`
      mutation createPort($input: CreatePortInput) {
        createPort(
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
  return result.data.createPort.port;
};

async function findComponent(args: SearchPrimitive): Promise<PortComponent[]> {
  let result = await apolloClient.query({ 
    query: gql`query findPortComponents($search: String!, $workspace: String) {
  findPortComponents(where: { search: $search, workspace: $workspace }) {
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
  return result.data.findPortComponents;
}

export const Port = {
  create,
  findComponent,
}
