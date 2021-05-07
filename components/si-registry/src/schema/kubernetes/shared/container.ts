import {
  PropObject,
  RegistryEntry,
  ValidatorKind,
} from "../../../registryEntry";
import { metadata } from "./objectMeta";

export const container: RegistryEntry["properties"] = [];

export const spec: PropObject = {
  type: "object",
  name: "spec",
  properties: container,
  link:
    "https://kubernetes.io/docs/reference/kubernetes-api/workload-resources/pod-v1/#PodSpec",
};
