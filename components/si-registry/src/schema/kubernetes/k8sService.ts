import {
  RegistryEntry,
  NodeKind,
  Arity,
  ValidatorKind,
  SchematicKind,
} from "../../registryEntry";

import { metadata } from "./shared/objectMeta";
import {
  apiVersion,
  kind,
  qualifications,
  actions,
  commands,
  code,
} from "./shared/standard";

const k8sService: RegistryEntry = {
  entityType: "k8sService",
  nodeKind: NodeKind.Concrete,
  code: code(),
  ui: {
    menu: [
      {
        name: "service",
        menuCategory: ["kubernetes"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["service"],
      },
    ],
  },
  inputs: [
    {
      name: "k8sNamespace",
      types: ["k8sNamespace"],
      edgeKind: "configures",
      arity: Arity.One,
    },
    {
      name: "k8sDeployment",
      types: ["k8sDeployment"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
  properties: [
    apiVersion("v1"),
    kind("Service"),
    metadata,
    {
      type: "object",
      name: "spec",
      properties: [
        {
          type: "string",
          name: "type",
          widget: {
            name: "select",
            options: {
              items: [
                { label: "ClusterIP", value: "ClusterIP" },
                { label: "ExternalName", value: "ExternalName" },
                { label: "NodePort", value: "NodePort" },
                { label: "LoadBalancer", value: "LoadBalancer" },
              ],
            },
          },
        },
        {
          type: "map",
          name: "selector",
          valueProperty: {
            type: "string",
          },
        },
        {
          type: "array",
          name: "ports",
          itemProperty: {
            type: "object",
            properties: [
              {
                type: "string",
                name: "name",
              },
              {
                type: "number",
                name: "port",
                validation: [
                  {
                    kind: ValidatorKind.Int,
                    options: { min: 0, max: 65536 },
                  },
                ],
              },
              {
                type: "string",
                name: "protocol",
                widget: {
                  name: "select",
                  options: {
                    items: [
                      { label: "TCP", value: "TCP" },
                      { label: "UDP", value: "UDP" },
                      { label: "SCTP", value: "SCTP" },
                    ],
                  },
                },
              },
              {
                type: "string",
                name: "targetPort",
              },
            ],
          },
        },
      ],
    },
  ],
  qualifications,
  actions,
  commands,
};

export default k8sService;
