import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
} from "../../registryEntry";
import { standardConceptInputs } from "../include/standardConceptInputs";

const service: RegistryEntry = {
  entityType: "service",
  nodeKind: NodeKind.Concept,
  ui: {
    menuCategory: MenuCategory.Application,
    menuDisplayName: "service",
    schematicKinds: [SchematicKind.Deployment],
  },
  inputs: [...standardConceptInputs],
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

export default service;
