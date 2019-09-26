import { SearchPrimitive } from "searchjs";
import gql from "graphql-tag";

import { apolloClient } from "@/apollo";
import { SshKeyComponent, SshKeyEntity } from "@/generated/graphql";

interface SshKeyCreate {
  constraints?: SearchPrimitive, 
  args?: {
    name?: string,
    description?: string,
    privateKey?: string,
    publicKey?: string,
  },
}

async function create(req?: SshKeyCreate): Promise<SshKeyEntity> {
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
      mutation createSshKey($input: CreateSshKeyInput) {
        createSshKey(
        input: $input
        ) {
          sshKey {
            id
            name
            description
            keyType
            keyFormat
            bits
            comment
            bubbleBabble
            fingerPrint
            randomArt
            privateKey
            publicKey
            component {
              id
              name
            }
          }
        }
      }`,
    variables: {input},
  });
  return result.data.createSshKey.sshKey;
};

async function findComponent(args: SearchPrimitive): Promise<SshKeyComponent[]> {
  let result = await apolloClient.query({ 
    query: gql`query findSshKeyComponents($search: String!, $workspace: String) {
  findSshKeyComponents(where: { search: $search, workspace: $workspace }) {
      id
      name
      description
      rawDataJson
      nodeType
      supportedActions
      serviceName
      keyType
      keyFormat
      bits
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
  return result.data.findSshKeyComponents;
}

export const SshKey = {
  create,
  findComponent,
}

