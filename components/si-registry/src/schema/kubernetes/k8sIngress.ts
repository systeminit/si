import {
  RegistryEntry,
  NodeKind,
  Arity,
  SchematicKind,
  ValidatorKind,
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

const k8sIngress: RegistryEntry = {
  entityType: "k8sIngress",
  nodeKind: NodeKind.Concept,
  code: code(),
  ui: {
    menu: [
      {
        name: "ingress",
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
  ],
  properties: [
    apiVersion("networking.k8s.io/v1"),
    kind("Ingress"),
    metadata,
    {
      type: "object",
      name: "spec",
      properties: [
        {
          type: "array",
          name: "rules",
          itemProperty: {
            type: "object",
            properties: [
              {
                type: "string",
                name: "host",
              },
              {
                type: "object",
                name: "http",
                properties: [
                  {
                    type: "array",
                    name: "paths",
                    itemProperty: {
                      type: "object",
                      properties: [
                        {
                          type: "string",
                          name: "path",
                        },
                        {
                          type: "string",
                          name: "pathType",
                          widget: {
                            name: "select",
                            options: {
                              items: [
                                { label: "Exact", value: "Exact" },
                                { label: "Prefix", value: "Prefix" },
                                {
                                  label: "ImplementationSpecific",
                                  value: "ImplementationSpecific",
                                },
                              ],
                            },
                          },
                        },
                        {
                          type: "object",
                          name: "backend",
                          properties: [
                            {
                              type: "object",
                              name: "service",
                              properties: [
                                {
                                  type: "string",
                                  name: "name",
                                },
                                {
                                  type: "object",
                                  name: "port",
                                  properties: [
                                    {
                                      type: "string",
                                      name: "name",
                                    },
                                    {
                                      type: "number",
                                      name: "number",
                                      validation: [
                                        {
                                          kind: ValidatorKind.Int,
                                          options: { min: 0, max: 65536 },
                                        },
                                      ],
                                    },
                                  ],
                                },
                              ],
                            },
                            {
                              type: "object",
                              name: "resource",
                              properties: [
                                {
                                  type: "string",
                                  name: "apiGroup",
                                },
                                {
                                  type: "string",
                                  name: "kind",
                                },
                                {
                                  type: "string",
                                  name: "name",
                                },
                              ],
                            },
                          ],
                        },
                      ],
                    },
                  },
                ],
              },
            ],
          },
        },
        {
          type: "object",
          name: "defaultBackend",
          properties: [
            {
              type: "object",
              name: "resource",
              properties: [
                {
                  type: "string",
                  name: "apiGroup",
                },
                {
                  type: "string",
                  name: "kind",
                },
                {
                  type: "string",
                  name: "name",
                },
              ],
            },
            {
              type: "object",
              name: "service",
              properties: [
                {
                  type: "string",
                  name: "name",
                },
                {
                  type: "object",
                  name: "port",
                  properties: [
                    {
                      type: "string",
                      name: "name",
                    },
                    {
                      type: "number",
                      name: "number",
                      validation: [
                        {
                          kind: ValidatorKind.Int,
                          options: { min: 0, max: 65536 },
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        },
        {
          type: "string",
          name: "ingressClassName",
        },
        {
          type: "array",
          name: "tls",
          itemProperty: {
            type: "object",
            properties: [
              {
                type: "array",
                name: "hosts",
                itemProperty: {
                  type: "string",
                },
              },
              {
                type: "string",
                name: "secretName",
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

export default k8sIngress;
