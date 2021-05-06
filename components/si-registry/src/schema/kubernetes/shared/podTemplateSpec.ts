import {
  PropObject,
  RegistryEntry,
  ValidatorKind,
} from "../../../registryEntry";
import { metadata } from "./objectMeta";
import { spec } from "./podSpec";

export const podTemplateSpec: RegistryEntry["properties"] = [metadata, spec];

export const template: PropObject = {
  type: "object",
  name: "template",
  properties: podTemplateSpec,
  link:
    "https://kubernetes.io/docs/reference/kubernetes-api/workload-resources/pod-template-v1/#PodTemplateSpec",
};
