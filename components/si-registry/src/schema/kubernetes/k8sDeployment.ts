import { RegistryEntry, NodeKind, Arity } from "../../registryEntry";

import { metadata } from "./shared/objectMeta";
import {
  apiVersion,
  kind,
  qualifications,
  actions,
  commands,
  ui,
  code,
} from "./shared/standard";
import { selector } from "./shared/labelSelector";
import { template } from "./shared/podTemplateSpec";

const k8sDeployment: RegistryEntry = {
  entityType: "k8sDeployment",
  nodeKind: NodeKind.Concrete,
  code: code(),
  ui: ui("k8sDeployment"),
  inputs: [
    {
      name: "dockerImage",
      types: ["dockerImage"],
      edgeKind: "configures",
      arity: Arity.Many,
    },
    {
      name: "k8sNamespace",
      types: ["k8sNamespace"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
  properties: [
    apiVersion("apps/v1"),
    kind("Deployment"),
    metadata,
    {
      type: "object",
      name: "spec",
      properties: [
        {
          type: "number",
          name: "replicas",
        },
        selector,
        template,
      ],
    },
  ],
  qualifications,
  actions,
  commands,
};

export default k8sDeployment;
