import {
  PropObject,
  RegistryEntry,
  ValidatorKind,
} from "../../../registryEntry";

export const labelSelector: RegistryEntry["properties"] = [
  {
    type: "array",
    name: "matchExpressions",
    itemProperty: {
      type: "object",
      properties: [
        {
          type: "string",
          name: "key",
        },
        {
          type: "string",
          name: "operator",
          widget: {
            name: "select",
            options: {
              items: [
                { label: "In", value: "In" },
                { label: "NotIn", value: "NotIn" },
                { label: "Exists", value: "Exists" },
                { label: "DoesNotExist", value: "DoesNotExist" },
              ],
            },
          },
          validation: [
            {
              kind: ValidatorKind.IsIn,
              values: ["In", "NotIn", "Exists", "DoesNotExist"],
            },
          ],
        },
        {
          type: "array",
          name: "values",
          itemProperty: {
            type: "string",
          },
        },
      ],
    },
  },
  {
    type: "map",
    name: "matchLabels",
    valueProperty: {
      type: "string",
    },
  },
];

export const selector: PropObject = {
  type: "object",
  name: "selector",
  properties: labelSelector,
  link:
    "https://kubernetes.io/docs/reference/kubernetes-api/common-definitions/label-selector/#LabelSelector",
};
