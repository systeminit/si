import {
  NodeKind,
  RegistryEntry,
  SchematicKind,
  ValidatorKind,
} from "../../registryEntry";

const dockerHubCredential: RegistryEntry = {
  entityType: "dockerHubCredential",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "docker hub credential",
        menuCategory: ["container", "docker"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["service"],
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
        secretKind: "dockerHub",
      },
      validation: [
        {
          kind: ValidatorKind.Required,
        },
      ],
    },
  ],
};

export default dockerHubCredential;
