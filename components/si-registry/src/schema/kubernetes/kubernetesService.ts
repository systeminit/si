import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";

const kubernetesService: RegistryEntry = {
  entityType: "kubernetesService",
  nodeKind: NodeKind.Implementation,
  ui: {
    menu: [
      {
        name: "kubernetes",
        menuCategory: ["implementation"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["service"],
      },
    ],
  },
  implements: ["service"],
  inputs: [
    {
      name: "k8sDeployment",
      types: ["k8sDeployment"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
    {
      name: "k8sService",
      types: ["k8sService"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
    {
      name: "k8sPod",
      types: ["k8sPod"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
  ],
  properties: [],
};

export default kubernetesService;
