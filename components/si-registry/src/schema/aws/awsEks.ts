import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
  Arity,
  //Arity,
} from "../../registryEntry";

const awsEks: RegistryEntry = {
  entityType: "awsEks",
  nodeKind: NodeKind.Implementation,
  ui: {
    menuCategory: MenuCategory.AWS,
    menuDisplayName: "awsEks",
    schematicKinds: [SchematicKind.Component],
  },
  implements: ["kubernetesCluster"],
  inputs: [
    {
      name: "torture",
      edgeKind: "configures",
      arity: Arity.Many,
      types: ["torture"],
    },
  ],
  properties: [],
};

export default awsEks;
