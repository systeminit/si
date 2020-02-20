import { SiComponent } from "@/registry/siComponent";

import getEntity from "./graphql/queries/getEntity.gql";
import listEntities from "./graphql/queries/listEntities.gql";
import pickComponent from "./graphql/queries/pickComponent.gql";
import streamEntityEvents from "./graphql/subscriptions/streamEntityEvents.gql";
import createEntity from "./graphql/mutations/createEntity.gql";

export const awsEksClusterRuntime = new SiComponent("awsEksClusterRuntime", {
  name: "AWS EKS Cluster Runtime",
  componentProperties: ["kubernetesVersion"],
  listHeaders: [
    { text: "Name", value: "name" },
    { text: "Kubernetes Version", value: "kubernetesVersion" },
    { text: "State", value: "state" },
  ],
  showActions: [{ displayName: "Add Node Group" }, { displayName: "Delete" }],
  showProperties: [
    { displayName: "ID", property: "id", showAs: "text" },
    {
      displayName: "Kubernetes Version",
      property: "kubernetesVersion",
      showAs: "text",
    },
  ],
  hints: [
    {
      constraintName: "kubernetesVersion",
      hintValue: '"1.14" | "1.13" | "1.12"',
    },
  ],
  getEntity,
  listEntities,
  pickComponent,
  streamEntityEvents,
  createEntity,
});
