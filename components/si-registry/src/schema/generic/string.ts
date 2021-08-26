import { RegistryEntry, SchematicKind, NodeKind } from "../../registryEntry";

const string: RegistryEntry = {
  entityType: "string",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "string",
        menuCategory: ["generic"],
        schematicKind: SchematicKind.Component,
      },
    ],
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "value",
    },
  ],
  inference: [
    {
      name: "copy",
      kind: "lambda",
      from: [{ targetEntity: true, data: { path: ["value"] } }],
      code: "firstEntity.properties.value",
    },
    {
      name: "upperCase",
      kind: "lambda",
      from: [{ targetEntity: true, data: { path: ["value"] } }],
      code: "_.toUpper(firstEntity.properties.value)",
    },
  ],
};

export default string;
