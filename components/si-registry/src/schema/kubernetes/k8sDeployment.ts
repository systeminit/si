import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";

const k8sDeployment: RegistryEntry = {
  entityType: "k8sDeployment",
  nodeKind: NodeKind.Concrete,
  ui: {
    menuCategory: MenuCategory.Kubernetes,
    menuDisplayName: "k8sDeployment",
    schematicKinds: [SchematicKind.Component],
  },
  inputs: [
    {
      name: "dockerImage",
      types: ["dockerImage"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
    {
      name: "k8sNamespace",
      types: ["k8sNamespace"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
  properties: [],
};

export default k8sDeployment;
