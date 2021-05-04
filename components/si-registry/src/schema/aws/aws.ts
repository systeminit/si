import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";

const aws: RegistryEntry = {
  entityType: "aws",
  nodeKind: NodeKind.Implementation,
  ui: {
    menuCategory: MenuCategory.AWS,
    menuDisplayName: "AWS",
    schematicKinds: [SchematicKind.Component],
  },
  implements: ["cloudProvider"],
  inputs: [
    {
      name: "awsRegion",
      types: ["awsRegion"],
      edgeKind: "configures",
      arity: Arity.One,
    },
    {
      name: "awsAccessKey",
      types: ["awsAccessKey"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
  properties: [],
};

export default aws;
