import { RegistryEntry } from "../../registryEntry";

const noCallbacks: RegistryEntry = {
  entityType: "noCallBacks",
  ui: {
    hidden: true,
  },
  properties: [
    {
      type: "string",
      name: "hearts Alive",
    },
  ],
};

export default noCallbacks;
