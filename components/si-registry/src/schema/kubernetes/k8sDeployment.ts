import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
  Arity,
  CodeKind,
} from "../../registryEntry";

const k8sDeployment: RegistryEntry = {
  entityType: "k8sDeployment",
  nodeKind: NodeKind.Concrete,
  ui: {
    menuCategory: MenuCategory.Kubernetes,
    menuDisplayName: "k8sDeployment",
    schematicKinds: [SchematicKind.Component],
  },
  code: { kind: CodeKind.YAML },
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
  properties: [
    { type: "string", name: "apiVersion" },
    { type: "string", name: "kind" },
    { type: "string", name: "spec" },
    { type: "string", name: "data" },
    { type: "string", name: "other" },
  ],
  actions: [{ name: "apply" }],
  commands: [
    {
      name: "apply",
      description: "kubectl apply",
    },
  ],
};

export default k8sDeployment;
