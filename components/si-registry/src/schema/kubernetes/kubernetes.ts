import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";
import { standardConceptInputs } from "../include/standardConceptInputs";

const kubernetesCluster: RegistryEntry = {
  entityType: "kubernetesCluster",
  nodeKind: NodeKind.Concept,
  ui: {
    menu: [
      {
        name: "kubernetes",
        menuCategory: ["compute"],
        schematicKind: SchematicKind.Deployment,
        rootEntityTypes: ["application"],
      },
    ],
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
