import {
  PropObject,
  RegistryEntry,
  ValidatorKind,
} from "../../../registryEntry";
import { metadataOptional } from "./objectMeta";
import { spec } from "./podSpec";

export const podTemplateSpec: RegistryEntry["properties"] = [
  metadataOptional,
  spec,
];

export const template: PropObject = {
  type: "object",
  name: "template",
  properties: podTemplateSpec,
  link:
    "https://kubernetes.io/docs/reference/kubernetes-api/workload-resources/pod-template-v1/#PodTemplateSpec",
};
