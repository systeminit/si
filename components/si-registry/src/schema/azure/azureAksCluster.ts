import {
  RegistryEntry,
  SchematicKind,
  NodeKind,
  Arity,
  WidgetSelectOptionsItems,
} from "../../registryEntry";

import _ from "lodash";

// taken from az aks get-versions -l centralus
const kubernetesVersions: Record<string, any> = {
  id:
    "/subscriptions/6602cdaf-54a1-43f3-afc6-d3efbf993c1c/providers/Microsoft.ContainerService/locations/centralus/orchestrators",
  name: "default",
  orchestrators: [
    {
      default: null,
      isPreview: null,
      orchestratorType: "Kubernetes",
      orchestratorVersion: "1.18.14",
      upgrades: [
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.18.17",
        },
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.19.7",
        },
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.19.9",
        },
      ],
    },
    {
      default: null,
      isPreview: null,
      orchestratorType: "Kubernetes",
      orchestratorVersion: "1.18.17",
      upgrades: [
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.19.7",
        },
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.19.9",
        },
      ],
    },
    {
      default: null,
      isPreview: null,
      orchestratorType: "Kubernetes",
      orchestratorVersion: "1.19.7",
      upgrades: [
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.19.9",
        },
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.20.2",
        },
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.20.5",
        },
      ],
    },
    {
      default: true,
      isPreview: null,
      orchestratorType: "Kubernetes",
      orchestratorVersion: "1.19.9",
      upgrades: [
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.20.2",
        },
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.20.5",
        },
      ],
    },
    {
      default: null,
      isPreview: null,
      orchestratorType: "Kubernetes",
      orchestratorVersion: "1.20.2",
      upgrades: [
        {
          isPreview: null,
          orchestratorType: "Kubernetes",
          orchestratorVersion: "1.20.5",
        },
      ],
    },
    {
      default: null,
      isPreview: null,
      orchestratorType: "Kubernetes",
      orchestratorVersion: "1.20.5",
      upgrades: null,
    },
  ],
  type: "Microsoft.ContainerService/locations/orchestrators",
};

export function generateLabels(): WidgetSelectOptionsItems {
  const items = _.map(kubernetesVersions.orchestrators, (v) => {
    return { label: `${v.orchestratorVersion}`, value: v.orchestratorVersion };
  });
  return { items };
}

const azureAksCluster: RegistryEntry = {
  entityType: "azureAksCluster",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "cluster",
        menuCategory: ["azure", "aks"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["kubernetesCluster"],
      },
    ],
  },
  inputs: [
    {
      name: "azureResourceGroup",
      edgeKind: "configures",
      arity: Arity.One,
      types: ["azureResourceGroup"],
    },
  ],
  properties: [
    {
      type: "string",
      name: "name",
    },
    {
      type: "string",
      name: "kubernetesVersion",
      defaultValue: "1.19.9",
      widget: {
        name: "select",
        options: generateLabels(),
      },
    },
  ],
};

export default azureAksCluster;
