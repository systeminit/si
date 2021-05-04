import {
  RegistryEntry,
  MenuCategory,
  ValidatorKind,
  SchematicKind,
  NodeKind,
  //Arity,
} from "../../registryEntry";

const awsAccessKey: RegistryEntry = {
  entityType: "awsAccessKey",
  nodeKind: NodeKind.Concrete,
  ui: {
    menuCategory: MenuCategory.AWS,
    menuDisplayName: "awsAccessKey",
    schematicKinds: [SchematicKind.Component],
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
    },
  ],
};

export default awsAccessKey;
