import { RegistryEntry, MenuCategory } from "../../registryEntry";

const service: RegistryEntry = {
  entityType: "service",
  ui: {
    menuCategory: MenuCategory.Application,
    menuDisplayName: "service",
    superNode: true
  },
  properties: [],
};

export default service;
