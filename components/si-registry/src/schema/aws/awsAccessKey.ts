import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  ValidatorKind,
  //Arity,
} from "../../registryEntry";

const awsAccessKey: RegistryEntry = {
  entityType: "awsAccessKey",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "access key",
        menuCategory: ["aws"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["cloudProvider"],
      },
    ],
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "secret",
      widget: {
        name: "selectFromSecret",
        secretKind: "awsAccessKey",
      },
      validation: [
        {
          kind: ValidatorKind.Required,
        },
      ],
    },
  ],
};

export default awsAccessKey;
