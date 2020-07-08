export function generateEntity(
  workspaceId: string,
  number: number,
): Record<string, any> {
  return {
    name: `poop${number}`,
    displayName: "poopy pants",
    description: "really poopy",
    workspaceId: workspaceId,
    properties: {
      kubernetesObject: {
        kind: "your butt",
        apiVersion: "1.0",
      },
    },
    constraints: { kubernetesVersion: "V1_15" },
  };
}

export function generateEntityFromVariables(
  workspaceId: string,
  number: number,
): Record<string, any> {
  return {
    name: `motherLoveBone${number}`,
    displayName: "Mother Love Bone",
    description: "Mother Love Bone",
    workspaceId: workspaceId,
    properties: {
      kubernetesObject: {
        apiVersion: "rr",
        kind: "rr",
        metadata: { name: "", labels: [] },
        spec: {
          replicas: 44,
          selector: { matchLabels: [] },
          template: {
            metadata: { name: "", labels: [] },
            spec: { containers: [] },
          },
        },
      },
    },
    constraints: {
      componentName: "",
      componentDisplayName: "",
      kubernetesVersion: "V1_15",
    },
  };
}
