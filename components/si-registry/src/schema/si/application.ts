import { RegistryEntry } from "../../registryEntry";

const application: RegistryEntry = {
  entityType: "application",
  ui: {
    hidden: true,
  },
  properties: [],
  actions: [
    {
      name: "deploy",
    },
  ],
};

export default application;
