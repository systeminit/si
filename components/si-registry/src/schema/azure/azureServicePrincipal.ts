import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  ValidatorKind,
  //Arity,
} from "../../registryEntry";

const azureServicePrincipal: RegistryEntry = {
  entityType: "azureServicePrincipal",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "service principal",
        menuCategory: ["azure"],
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
        secretKind: "azureServicePrincipal",
      },
      validation: [
        {
          kind: ValidatorKind.Required,
        },
      ],
    },
  ],
};

export default azureServicePrincipal;
