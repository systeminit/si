import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";

const aws: RegistryEntry = {
  entityType: "aws",
  nodeKind: NodeKind.Implementation,
  ui: {
    menu: [
      {
        name: "aws",
        menuCategory: ["implementation"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["cloudProvider"],
      },
    ],
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
