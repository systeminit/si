import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
  //Arity,
} from "../../registryEntry";

const azureAks: RegistryEntry = {
  entityType: "azureAks",
  nodeKind: NodeKind.Implementation,
  ui: {
    menu: [
      {
        name: "aks",
        menuCategory: ["implementation", "azure"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["kubernetesCluster"],
      },
    ],
  },
  implements: ["kubernetesCluster"],
  inputs: [
    {
      name: "azureAksCluster",
      edgeKind: "configures",
      arity: Arity.One,
      types: ["azureAksCluster"],
    },
  ],
  properties: [],
};

export default azureAks;
