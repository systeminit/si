import {
  RegistryEntry,
  MenuCategory,
  SchematicKind,
  NodeKind,
  //Arity,
} from "../../registryEntry";

const awsEks: RegistryEntry = {
  entityType: "awsEks",
  nodeKind: NodeKind.Implementation,
  ui: {
    menuCategory: MenuCategory.AWS,
    menuDisplayName: "awsEks",
    schematicKinds: [SchematicKind.Component],
  },
  implements: ["kubernetes"],
  inputs: [],
  properties: [],
};

export default awsEks;
