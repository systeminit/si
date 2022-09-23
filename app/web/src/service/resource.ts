import { useObservable } from "@vueuse/rxjs";
import { ResourceHealth, ResourceStatus } from "@/api/sdf/dal/resource";
import { listByComponent } from "./resource/list_by_component";

export interface Confirmation {
  title: string;
  health: ResourceHealth;
  link?: string;
  description?: string;
  output?: string[];
}

export enum ComponentSchema {
  DockerImage = "docker_image",
  DockerHubCredential = "docker_hub_credential",
  KubernetesNamespace = "kubernetes_namespace",
  KubernetesDeployment = "kubernetes_deployment",
  CoreOsButane = "coreos_butane",
}

export type Resource = {
  id: number;
  name: string;
  kind: string;
  health: ResourceHealth;
  status: ResourceStatus;
  confirmations: Confirmation[];
};

function useResourceSummary() {
  return useObservable(listByComponent());
}

export const ResourceService = {
  useResourceSummary,
};
