import { RegistryEntry, NodeKind } from "../../registryEntry";

const system: RegistryEntry = {
  entityType: "system",
  nodeKind: NodeKind.Concept,
  ui: {
    hidden: true,
  },
  inputs: [],
  properties: [],
};

export default system;
