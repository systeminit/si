import {
  RegistryEntry,
  MenuCategory,
  ValidatorKind,
  SchematicKind,
  NodeKind,
  //Arity,
} from "../../registryEntry";

const awsEksCluster: RegistryEntry = {
  entityType: "awsEksCluster",
  nodeKind: NodeKind.Concrete,
  ui: {
    menuCategory: MenuCategory.AWS,
    menuDisplayName: "awsEksCluster",
    schematicKinds: [SchematicKind.Component],
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "name",
    },
    {
      type: "string",
      name: "kubernetesVersion",
      defaultValue: "1.19",
      widget: {
        name: "select",
        options: {
          items: [
            { value: "1.19", label: "1.19" },
            { value: "1.18", label: "1.18" },
            { value: "1.17", label: "1.17" },
            { value: "1.16", label: "1.16" },
            { value: "1.15", label: "1.15" },
          ],
        },
      },
    },
  ],
};

export default awsEksCluster;
