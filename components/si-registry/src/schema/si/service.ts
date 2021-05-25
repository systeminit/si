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
    {
      type: "array",
      name: "healthChecks",
      itemProperty: {
        type: "object",
        properties: [
          {
            type: "string",
            name: "protocol",
            widget: {
              name: "select",
              options: {
                items: [
                  { label: "HTTP", value: "HTTP" },
                  { label: "HTTPS", value: "HTTPS" },
                  { label: "TCP", value: "TCP" },
                  { label: "UDP", value: "UDP" },
                ],
              },
            },
          },
          {
            type: "string",
            name: "host",
          },
          {
            type: "string",
            name: "port",
          },
          {
            type: "string",
            name: "path",
          },
        ],
      },
    },
  ],
  actions: [{ name: "deploy" }],
};

export default service;
