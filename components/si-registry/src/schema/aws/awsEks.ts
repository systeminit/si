import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
  //Arity,
} from "../../registryEntry";

const awsEks: RegistryEntry = {
  entityType: "awsEks",
  nodeKind: NodeKind.Implementation,
  ui: {
    menu: [
      {
        name: "eks",
        menuCategory: ["implementation", "aws"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["kubernetesCluster"],
      },
    ],
  },
  implements: ["kubernetesCluster"],
  inputs: [
    {
      name: "awsEksCluster",
      edgeKind: "configures",
      arity: Arity.One,
      types: ["awsEksCluster"],
    },
  ],
  properties: [],
};

export default awsEks;
