import {
  RegistryEntry,
  ValidatorKind,
  SchematicKind,
  NodeKind,
  CodeKind,
  Arity,
} from "../../registryEntry";

const yamlNumbers: RegistryEntry = {
  entityType: "yamlNumbers",
  nodeKind: NodeKind.Concrete,
  code: { kind: CodeKind.YAML },
  ui: {
    hidden: true,
  },
  inputs: [],
  commands: [],
  actions: [],
  properties: [
    {
      type: "string",
      name: "stringType",
    },
    {
      type: "number",
      name: "numberType",
    },
    {
      type: "map",
      name: "stringMap",
      valueProperty: {
        type: "string",
      },
    },
    {
      type: "map",
      name: "numberMap",
      valueProperty: {
        type: "number",
      },
    },
    {
      type: "array",
      name: "stringArray",
      itemProperty: {
        type: "string",
      },
    },
    {
      type: "array",
      name: "numberArray",
      itemProperty: {
        type: "number",
      },
    },
    {
      type: "object",
      name: "nestedObject",
      properties: [
        {
          type: "string",
          name: "objectString",
        },
        {
          type: "number",
          name: "objectNumber",
        },
        {
          type: "map",
          name: "objectStringMap",
          valueProperty: {
            type: "string",
          },
        },
        {
          type: "map",
          name: "objectNumberMap",
          valueProperty: {
            type: "number",
          },
        },
        {
          type: "array",
          name: "objectStringArray",
          itemProperty: {
            type: "string",
          },
        },
        {
          type: "array",
          name: "objectNumberArray",
          itemProperty: {
            type: "number",
          },
        },
        {
          type: "array",
          name: "objectArrayArray",
          itemProperty: {
            type: "array",
            itemProperty: {
              type: "array",
              itemProperty: {
                type: "object",
                properties: [
                  {
                    type: "string",
                    name: "deeplyNestedString",
                  },
                  {
                    type: "number",
                    name: "deeplyNestedNumber",
                  },
                  {
                    type: "array",
                    name: "deeplyNestedArrayNumber",
                    itemProperty: {
                      type: "number",
                    },
                  },
                ],
              },
            },
          },
        },
      ],
    },
  ],
};

export default yamlNumbers;
