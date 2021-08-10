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
