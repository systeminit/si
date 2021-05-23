import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";

const azure: RegistryEntry = {
  entityType: "azure",
  nodeKind: NodeKind.Implementation,
  ui: {
    menu: [
      {
        name: "azure",
        menuCategory: ["implementation"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["cloudProvider"],
      },
    ],
  },
  implements: ["cloudProvider"],
  inputs: [
    {
      name: "azureServicePrincipal",
      types: ["azureServicePrincipal"],
      edgeKind: "configures",
      arity: Arity.One,
    },
    {
      name: "azureLocation",
      types: ["azureLocation"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
  properties: [],
};

export default azure;
