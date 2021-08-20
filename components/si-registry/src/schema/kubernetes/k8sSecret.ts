import {
  RegistryEntry,
  NodeKind,
  Arity,
  SchematicKind,
} from "../../registryEntry";

import { metadata } from "./shared/objectMeta";
import {
  apiVersion,
  kind,
  qualifications,
  actions,
  commands,
  code,
} from "./shared/standard";

const k8sSecret: RegistryEntry = {
  entityType: "k8sSecret",
  nodeKind: NodeKind.Concrete,
  code: code(),
  ui: {
    menu: [
      {
        name: "secret",
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
    {
      name: "dockerHubCredential",
      types: ["dockerHubCredential"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
  properties: [
    apiVersion("v1"),
    kind("Secret"),
    metadata,
    {
      type: "string",
      name: "type",
      widget: {
        name: "select",
        options: {
          items: [
            { label: "Opaque", value: "Opaque" },
            {
              label: "kubernetes.io/service-account-token",
              value: "kubernetes.io/service-account-token",
            },
            {
              label: "kubernetes.io/dockerconfigjson",
              value: "kubernetes.io/dockerconfigjson",
            },
            {
              label: "kubernetes.io/basic-auth",
              value: "kubernetes.io/basic-auth",
            },
            {
              label: "kubernetes.io/ssh-auth",
              value: "kubernetes.io/ssh-auth",
            },
            { label: "kubernetes.io/tls", value: "kubernetes.io/tls" },
            {
              label: "bootstrap.kubernetes.io/token",
              value: "bootstrap.kubernetes.io/token",
            },
          ],
        },
      },
    },

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

export default k8sSecret;
