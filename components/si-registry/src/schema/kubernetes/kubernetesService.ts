import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
} from "../../registryEntry";

const kubernetesService: RegistryEntry = {
  entityType: "kubernetesService",
  nodeKind: NodeKind.Implementation,
  ui: {
    menu: [
      {
        name: "kubernetes",
        menuCategory: ["implementation"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["service"],
      },
    ],
  },
  implements: ["service"],
  inputs: [
    {
      name: "k8sDeployment",
      types: ["k8sDeployment"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
    {
      name: "k8sService",
      types: ["k8sService"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
    {
      name: "k8sPod",
      types: ["k8sPod"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
  ],
  properties: [
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
};

export default kubernetesService;
