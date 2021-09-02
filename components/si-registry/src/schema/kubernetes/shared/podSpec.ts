import {
  PropObject,
  RegistryEntry,
  ValidatorKind,
} from "../../../registryEntry";

export const podSpec: RegistryEntry["properties"] = [
  {
    type: "array",
    name: "containers",
    itemProperty: {
      type: "object",
      properties: [
        { type: "string", name: "name" },
        {
          type: "string",
          name: "image",
        },
        {
          type: "array",
          name: "env",
          itemProperty: {
            type: "object",
            properties: [
              {
                type: "string",
                name: "name",
              },
              {
                type: "string",
                name: "value",
              },
              {
                type: "object",
                name: "valueFrom",
                properties: [
                  {
                    type: "object",
                    name: "secretKeyRef",
                    properties: [
                      {
                        type: "string",
                        name: "name",
                      },
                      {
                        type: "string",
                        name: "key",
                      },
                      {
                        type: "boolean",
                        name: "optional",
                      },
                    ],
                  },
                  {
                    type: "object",
                    name: "configMapRef",
                    properties: [
                      {
                        type: "string",
                        name: "name",
                      },
                      {
                        type: "string",
                        name: "key",
                      },
                      {
                        type: "boolean",
                        name: "optional",
                      },
                    ],
                  },
                  {
                    type: "object",
                    name: "resourceFieldRef",
                    properties: [
                      {
                        type: "string",
                        name: "containerName",
                      },
                      {
                        type: "string",
                        name: "resource",
                      },
                      {
                        type: "string",
                        name: "divisor",
                      },
                    ],
                  },
                  {
                    type: "object",
                    name: "fieldRef",
                    properties: [
                      {
                        type: "string",
                        name: "apiVersion",
                      },
                      {
                        type: "string",
                        name: "fieldPath",
                      },
                    ],
                  },
                ],
              },
            ],
            editPartials: [
              {
                kind: "item",
                name: "env.value",
                propertyPaths: [["name"], ["value"]],
              },
              {
                kind: "category",
                name: "env.valueFrom",
                items: [
                  {
                    kind: "item",
                    name: "env.valueFrom.configMapRef",
                    propertyPaths: [["name"], ["valueFrom", "configMapRef"]],
                  },
                  {
                    kind: "item",
                    name: "env.valueFrom.fieldRef",
                    propertyPaths: [["name"], ["valueFrom", "fieldRef"]],
                  },
                  {
                    kind: "item",
                    name: "env.valueFrom.resourceFieldRef",
                    propertyPaths: [
                      ["name"],
                      ["valueFrom", "resourceFieldRef"],
                    ],
                  },
                  {
                    kind: "item",
                    name: "env.valueFrom.secretKeyRef",
                    propertyPaths: [["name"], ["valueFrom", "secretKeyRef"]],
                  },
                ],
              },
            ],
          },
        },
        {
          type: "string",
          name: "imagePullPolicy",
          widget: {
            name: "select",
            options: {
              items: [
                { label: "Always", value: "Always" },
                { label: "Never", value: "Never" },
                { label: "IfNotPresent", value: "IfNotPresent" },
              ],
            },
          },
          validation: [
            {
              kind: ValidatorKind.Required,
            },
          ],
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
                name: "containerPort",
                validation: [
                  {
                    kind: ValidatorKind.Int,
                    options: { min: 0, max: 65536 },
                  },
                ],
              },
              {
                type: "string",
                name: "hostIP",
              },
              {
                type: "number",
                name: "hostPort",
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
            ],
          },
        },
        {
          type: "array",
          name: "volumeMounts",
          itemProperty: {
            type: "object",
            properties: [
              {
                type: "string",
                name: "name",
              },
              {
                type: "string",
                name: "mountPath",
              },
            ],
          },
        },
      ],
    },
  },
  {
    type: "array",
    name: "volumes",
    itemProperty: {
      type: "object",
      properties: [
        { type: "string", name: "name" },
        {
          type: "object",
          name: "configMap",
          properties: [{ type: "string", name: "name" }],
        },
      ],
    },
  },
  {
    type: "array",
    name: "imagePullSecrets",
    itemProperty: {
      type: "object",
      properties: [
        {
          type: "string",
          name: "name",
        },
      ],
    },
  },
];

export const spec: PropObject = {
  type: "object",
  name: "spec",
  properties: podSpec,
  link:
    "https://kubernetes.io/docs/reference/kubernetes-api/workload-resources/pod-v1/#PodSpec",
};
