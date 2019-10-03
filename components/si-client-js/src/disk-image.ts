import { SearchPrimitive } from "searchjs";
import gql from "graphql-tag";

import { apolloClient } from "@/apollo";
import { DiskImageEntity, DiskImageComponent } from "@/generated/graphql";

interface DiskImageCreate {
  constraints?: SearchPrimitive, 
  args?: {
    name?: string,
    description?: string,
  },
}

async function create(req?: DiskImageCreate): Promise<DiskImageEntity> {
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
      mutation createDiskImage($input: CreateDiskImageInput) {
        createDiskImage(
          input: $input
        ) {
          diskImage {
            id
            name
            description
            format
            operatingSystem {
              id
              name
              description
              operatingSystemName
              operatingSystemVersion
              platform
              platformVersion
            }
            component {
              id
              name
            }
          }
        }
      }`,
    variables: {input},
  });
  return result.data.createDiskImage.diskImage;
};

async function findComponent(args: SearchPrimitive): Promise<DiskImageComponent[]> {
  let result = await apolloClient.query({ 
    query: gql`query findDiskImageComponents($search: String!, $workspace: String) {
      findDiskImageComponents(where: { search: $search, workspace: $workspace }) {
        id
        name
        description
        format
        operatingSystem {
          id
          name
          description
          operatingSystemName
          operatingSystemVersion
          platform
          platformVersion
        }
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
  return result.data.findDiskImageComponents;
}

export const DiskImage = {
  create,
  findComponent,
}
