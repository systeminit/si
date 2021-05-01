import { RegistryEntry, NodeKind } from "../../registryEntry";

const application: RegistryEntry = {
  entityType: "application",
  nodeKind: NodeKind.Concept,
  ui: {
    hidden: true,
  },
  inputs: [],
  properties: [],
  actions: [
    {
      name: "deploy",
    },
  ],
};

export default application;
