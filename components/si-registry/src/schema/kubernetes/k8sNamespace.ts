import { RegistryEntry, NodeKind, SchematicKind } from "../../registryEntry";

import { metadata } from "./shared/objectMeta";
import {
  apiVersion,
  kind,
  qualifications,
  actions,
  commands,
  code,
} from "./shared/standard";

const k8sNamespace: RegistryEntry = {
  entityType: "k8sNamespace",
  nodeKind: NodeKind.Concrete,
  code: code(),
  ui: {
    menu: [
      {
        name: "namespace",
        menuCategory: ["kubernetes"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["service"],
      },
    ],
  },
  inputs: [],
  properties: [apiVersion("v1"), kind("Namespace"), metadata],
  qualifications,
  actions,
  commands,
};

export default k8sNamespace;
