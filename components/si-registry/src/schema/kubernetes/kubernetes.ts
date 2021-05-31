import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
  ValidatorKind,
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
  omitOutputsInSchematic: [SchematicKind.Component],
  properties: [
    {
      type: "string",
      name: "implementation",
      widget: {
        name: "selectFromInput",
        inputName: "implementations",
      },
      validation: [
        {
          kind: ValidatorKind.Required,
        },
      ],
    },
  ],
};

export default kubernetesCluster;
