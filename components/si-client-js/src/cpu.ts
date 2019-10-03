import { SearchPrimitive } from "searchjs";
import gql from "graphql-tag";

import { apolloClient } from "@/apollo";
import { CpuComponent, CpuEntity } from "@/generated/graphql";

export interface CpuCreate {
  constraints?: SearchPrimitive, 
  args?: {
    name?: string,
    description?: string,
  },
}

async function create(req?: CpuCreate): Promise<CpuEntity> {
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
      mutation createCpu($input: CreateCpuInput) {
        createCpu(
        input: $input
        ) {
          cpu {
            id
            name
            description
            cores
            baseFreqMHz
            allCoreTurboFreqMHz
            singleCoreTurboFreqMHz
            architecture
            manufacturer
            component {
              id
              name
            }
          }
        }
      }`,
    variables: {input},
  });
  return result.data.createCpu.cpu;
};

async function findComponent(args: SearchPrimitive): Promise<CpuComponent[]> {
  let result = await apolloClient.query({ 
    query: gql`query findCpuComponents($search: String!, $workspace: String) {
  findCpuComponents(where: { search: $search, workspace: $workspace }) {
      id
      name
      description
      rawDataJson
      nodeType
      supportedActions
      cores
      baseFreqMHz
      allCoreTurboFreqMHz
      singleCoreTurboFreqMHz
      architecture
      manufacturer
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
  return result.data.findCpuComponents;
}

export const Cpu = {
  create,
  findComponent,
}
