import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  //Arity,
} from "../../registryEntry";
import { generateLabels } from "./azureLocation";

const azureResourceGroup: RegistryEntry = {
  entityType: "azureResourceGroup",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "resource group",
        menuCategory: ["azure"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["kubernetesCluster", "azure"],
      },
    ],
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "name",
    },
    {
      type: "string",
      name: "location",
      widget: {
        name: "select",
        options: generateLabels(),
      },
    },
    {
      type: "map",
      name: "tags",
      valueProperty: {
        type: "string",
      },
    },
  ],
};

export default azureResourceGroup;
