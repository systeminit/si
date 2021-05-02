import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";
import { standardConceptInputs } from "../include/standardConceptInputs";

const kubernetesCluster: RegistryEntry = {
  entityType: "kubernetesCluster",
  nodeKind: NodeKind.Concept,
  ui: {
    menuCategory: MenuCategory.Application,
    menuDisplayName: "kubernetes",
    schematicKinds: [SchematicKind.Deployment],
  },
  inputs: [
    ...standardConceptInputs,
    {
      name: "service",
      types: ["service"],
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
};

export default kubernetesCluster;
