import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
} from "../../registryEntry";
import { standardConceptInputs } from "../include/standardConceptInputs";

const service: RegistryEntry = {
  entityType: "service",
  ui: {
    menuCategory: MenuCategory.Application,
    menuDisplayName: "service",
    schematicKinds: [SchematicKind.Deployment],
  },
  inputs: [...standardConceptInputs],
  properties: [],
};

export default service;
