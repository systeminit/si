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
  properties: [],
};

export default service;
