import { RegistryEntry, NodeKind } from "../../registryEntry";

const noCallbacks: RegistryEntry = {
  entityType: "noCallBacks",
  nodeKind: NodeKind.Concept,
  ui: {
    hidden: true,
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "hearts Alive",
    },
  ],
};

export default noCallbacks;
