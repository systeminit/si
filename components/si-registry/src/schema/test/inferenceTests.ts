import {
  RegistryEntry,
  ValidatorKind,
  SchematicKind,
  NodeKind,
  CodeKind,
  Arity,
} from "../../registryEntry";

const inferenceTests: RegistryEntry = {
  entityType: "inferenceTests",
  nodeKind: NodeKind.Concrete,
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
      type: "string",
      name: "nameField",
      inference: {
        select: {
          single: true,
          default: "fromName",
        },
        functions: [
          {
            name: "fromName",
            kind: "lambda",
            from: [{ targetEntity: true, data: { name: true } }],
            code: "firstEntity.name",
          },
          {
            name: "fromEntity",
            kind: "lambda",
            userProvides: [
              {
                required: true,
                from: [
                  {
                    kind: "entityId",
                    data: [{ name: true }],
                  },
                ],
              },
            ],
            code: "fistEntity.name",
          },
        ],
      },
    },
    {
      type: "number",
      name: "numberType",
    },
    {
      type: "boolean",
      name: "booleanType",
    },
    {
      type: "object",
      name: "objectWithScalarValues",
      properties: [
        { type: "string", name: "stringValue" },
        { type: "number", name: "numberValue" },
        { type: "boolean", name: "booleanValue" },
      ],
    },
    {
      type: "map",
      name: "mapWithStringValues",
      valueProperty: {
        type: "string",
      },
    },
    {
      type: "map",
      name: "mapWithObjectValues",
      valueProperty: {
        type: "object",
        properties: [{ type: "string", name: "stringValue" }],
      },
    },
    {
      type: "array",
      name: "arrayWithStringValues",
      itemProperty: {
        type: "string",
      },
    },
    {
      type: "array",
      name: "complexArray",
      itemProperty: {
        type: "object",
        properties: [
          {
            type: "array",
            name: "nestedArray0",
            itemProperty: {
              type: "array",
              itemProperty: {
                type: "map",
                valueProperty: {
                  type: "string",
                },
              },
            },
          },
        ],
      },
    },
  ],
};

export default inferenceTests;
