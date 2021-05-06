import { RegistryEntry, NodeKind } from "../../registryEntry";

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

const k8sNamespace: RegistryEntry = {
  entityType: "k8sNamespace",
  nodeKind: NodeKind.Concrete,
  code: code(),
  ui: ui("k8sNamespace"),
  inputs: [],
  properties: [apiVersion("v1"), kind("Namespace"), metadata],
  qualifications,
  actions,
  commands,
};

export default k8sNamespace;
