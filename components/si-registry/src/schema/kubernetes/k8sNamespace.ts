import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
} from "../../registryEntry";

const k8sNamespace: RegistryEntry = {
  entityType: "k8sNamespace",
  nodeKind: NodeKind.Concrete,
  ui: {
    menuCategory: MenuCategory.Kubernetes,
    menuDisplayName: "k8sNamespace",
    schematicKinds: [SchematicKind.Component],
  },
  inputs: [],
  properties: [],
};

export default k8sNamespace;
