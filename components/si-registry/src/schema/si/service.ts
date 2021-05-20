import { RegistryEntry, SchematicKind, NodeKind } from "../../registryEntry";
import { standardConceptInputs } from "../include/standardConceptInputs";

const service: RegistryEntry = {
  entityType: "service",
  nodeKind: NodeKind.Concept,
  ui: {
    menu: [
      {
        name: "service",
        menuCategory: ["application"],
        schematicKind: SchematicKind.Deployment,
        rootEntityTypes: ["application"],
      },
    ],
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
