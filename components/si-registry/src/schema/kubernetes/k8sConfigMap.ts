import {
  RegistryEntry,
  NodeKind,
  Arity,
  SchematicKind,
} from "../../registryEntry";

import { metadata } from "./shared/objectMeta";
import { apiVersion, kind, actions, commands, code } from "./shared/standard";

const k8sConfigMap: RegistryEntry = {
  entityType: "k8sConfigMap",
  nodeKind: NodeKind.Concrete,
  code: code(),
  ui: {
    menu: [
      {
        name: "configMap",
        menuCategory: ["kubernetes"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["service"],
      },
    ],
  },
  inputs: [
    {
      name: "k8sNamespace",
      types: ["k8sNamespace"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
  properties: [
    apiVersion("v1"),
    kind("ConfigMap"),
    metadata,
    {
      type: "map",
      name: "data",
      valueProperty: {
        type: "string",
        widget: {
          name: "textArea",
        },
      },
    },
  ],
  actions,
  commands,
};

export default k8sConfigMap;

// ---
//     apiVersion: v1
// kind: ConfigMap
// metadata:
// name: otelcol
// data:
// config.yaml: |
