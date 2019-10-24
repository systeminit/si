import { SearchPrimitive } from "searchjs";
import gql from "graphql-tag";

import { apolloClient } from "@/apollo";
import { OperatingSystemComponent, OperatingSystemEntity } from "@/generated/graphql";

export interface OperatingSystemCreate {
  constraints?: SearchPrimitive, 
  args?: {
    name?: string,
    description?: string,
  },
}

async function create(req?: OperatingSystemCreate): Promise<OperatingSystemEntity> {
  if (!req) {
    req = {};
  }
  const input = {
    constraints: JSON.stringify(req.constraints),
    args: req.args,
    workspace: "1640dd40-f388-42e2-ab0a-7fafd36f8173",
  };
  let result = await apolloClient.mutate({
    mutation: gql`
      mutation createOperatingSystem($input: CreateOperatingSystemInput) {
        createOperatingSystem(
          input: $input
        ) {
          operatingSystem {
            id
            name
            description
            operatingSystemName
            operatingSystemVersion
            operatingSystemRelease
            platform
            platformVersion
            platformRelease
            architecture
            component {
              id
              name
            }
          }
        }
      }`,
    variables: {input},
  });
  return result.data.createOperatingSystem.operatingSystem;
};

async function findComponent(args: SearchPrimitive): Promise<OperatingSystemComponent[]> {
  let result = await apolloClient.query({ 
    query: gql`query findOperatingSystemComponents($search: String!, $workspace: String) {
      findOperatingSystemComponents(where: { search: $search, workspace: $workspace }) {
        id
        name
        description
        operatingSystemName
        operatingSystemVersion
        operatingSystemRelease
        platform
        platformVersion
        platformRelease
        architecture
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
  return result.data.findOperatingSystemComponents;
}

export const OperatingSystem = {
  create,
  findComponent,
}


