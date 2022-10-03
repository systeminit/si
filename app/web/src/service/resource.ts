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
  DockerImage = "Docker Image",
  DockerHubCredential = "Docker Hub Credential",
  KubernetesNamespace = "Kubernetes Namespace",
  KubernetesDeployment = "Kubernetes Deployment",
  CoreOsButane = "Butane",
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
