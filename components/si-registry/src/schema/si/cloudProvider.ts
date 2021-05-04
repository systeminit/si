import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";
import { onlyImplementation } from "../include/standardConceptInputs";

const cloudProvider: RegistryEntry = {
  entityType: "cloudProvider",
  nodeKind: NodeKind.Concept,
  ui: {
    menuCategory: MenuCategory.Application,
    menuDisplayName: "cloudProvider",
    schematicKinds: [SchematicKind.Deployment],
  },
  inputs: [
    ...onlyImplementation,
    {
      name: "kubernetesCluster",
      types: ["kubernetesCluster"],
      edgeKind: "deployment",
      arity: Arity.Many,
    },
  ],
  properties: [
    {
      type: "string",
      name: "implementation",
      widget: {
        name: "selectFromInput",
        inputName: "implementations",
      },
    },
  ],
  actions: [{ name: "deploy" }],
};

export default cloudProvider;
