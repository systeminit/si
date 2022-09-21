import { useObservable } from "@vueuse/rxjs";
import { ReplaySubject } from "rxjs";
import HealthIconVue, { Health } from "@/molecules/HealthIcon.vue";
import { ComponentListItem } from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";

export interface Confirmation {
  title: string;
  health: Health;
  link?: string;
  description?: string;
  output?: string[];
}

export interface ConfirmationSummaryForComponent {
  id: number;
  name: string;
  type: ComponentType;
  health: Health;
}

export interface ConfirmationSummary {
  components: Array<ConfirmationSummaryForComponent>;
}

export enum ComponentType {
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
  health: Health;
  confirmations: Confirmation[];
};

export const fakeResources = (
  component: ComponentListItem,
): Array<Resource> => {
  const data = {
    docker_image: [
      {
        id: 1,
        name: "whiskers",
        kind: "docker image",
        health: "Ok" as Health,
        confirmations: [
          {
            title: "test confirmation 1",
            health: "Ok" as Health,
            link: "idk",
            description: "this is just a test",
            output: [],
          },
        ],
      },
    ],
    docker_hub_credential: [
      {
        id: 1,
        name: "fake credential",
        kind: "docker hub credential",
        health: "Error" as Health,
        confirmations: [
          {
            title: "test confirmation 2",
            health: "Error" as Health,
            link: "idk",
            description: "this is just a test",
          },
        ],
      },
    ],
    kubernetes_namespace: [
      {
        id: 1,
        name: "my k8s namespace",
        kind: "k8s namespace",
        health: "Unknown" as Health,
        confirmations: [
          {
            title: "test confirmation 3",
            health: "Unknown" as Health,
            link: "idk",
            description: "this is just a test",
          },
        ],
      },
    ],
    kubernetes_deployment: [
      {
        id: 1,
        name: "let's deploy to k8s",
        kind: "k8s deployment",
        health: "Warning" as Health,
        confirmations: [
          {
            title: "test confirmation 4",
            health: "Warning" as Health,
            link: "idk",
            description: "this is just a test",
          },
        ],
      },
    ],
    coreos_butane: [
      {
        id: 1,
        name: "idk butane or something",
        kind: "coreos butane",
        health: "Ok" as Health,
        confirmations: [
          {
            title: "test confirmation 5",
            health: "Ok" as Health,
            link: "idk",
            description: "this is just a test",
          },
        ],
      },
    ],
    unknown: [
      {
        id: 2,
        kind: "unknown",
        name: "other resource",
        health: "Error" as Health,
        confirmations: [
          {
            title: "test confirmation 6",
            health: "Error" as Health,
            link: "idk",
            description: "this is just a test",
          },
        ],
      },
    ],
  };
  if (component.type) return data[component.type];
  else return data.unknown;
};

const mockComponentData: ConfirmationSummary = {
  components: [
    {
      id: 1,
      name: "mock component 1",
      type: ComponentType.DockerImage,
      health: "Ok",
    },
    {
      id: 2,
      name: "mock component 2",
      type: ComponentType.DockerHubCredential,
      health: "Error",
    },
    {
      id: 3,
      name: "mock component 3",
      type: ComponentType.KubernetesNamespace,
      health: "Unknown",
    },
    {
      id: 4,
      name: "mock component 4",
      type: ComponentType.KubernetesDeployment,
      health: "Warning",
    },
    {
      id: 5,
      name: "mock component 5",
      type: ComponentType.CoreOsButane,
      health: "Ok",
    },
  ],
};

const mockComponentData$ = new ReplaySubject<ConfirmationSummary>();
mockComponentData$.next(mockComponentData);

function useConfirmationSummary() {
  return useObservable(mockComponentData$); // TODO(wendy) - replace mock data with my own endpoint
}

export const ConfirmationService = {
  useConfirmationSummary,
};
