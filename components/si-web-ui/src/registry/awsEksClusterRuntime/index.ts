import { SiComponent } from "@/registry/siComponent";

import getEntity from "./graphql/queries/getEntity.gql";
import listEntities from "./graphql/queries/listEntities.gql";
import pickComponent from "./graphql/queries/pickComponent.gql";
import streamEntityEvents from "./graphql/subscriptions/streamEntityEvents.gql";

import createEntity from "./graphql/mutations/createEntity.gql";
import syncEntity from "./graphql/mutations/syncEntity.gql";

import listEntityEvents from "./graphql/queries/listEntityEvents.gql";

import addNodegroup from "./graphql/mutations/addNodegroup.gql";

export const awsEksClusterRuntime = new SiComponent("awsEksClusterRuntime", {
  name: "AWS EKS Cluster Runtime",
  componentProperties: ["kubernetesVersion"],
  listHeaders: [
    { text: "Name", value: "name" },
    { text: "Kubernetes Version", value: "kubernetesVersion" },
    { text: "Cluster Status", value: "clusterStatus" },
    { text: "Nodegroup Status", value: "nodegroupStatus" },
    { text: "State", value: "state" },
  ],
  showActions: [
    { displayName: "Add Nodegroup", mutation: addNodegroup },
    { displayName: "Delete" },
    { displayName: "Sync", mutation: syncEntity },
  ],
  showProperties: [
    { displayName: "ID", property: "id", showAs: "text" },
    {
      displayName: "Kubernetes Version",
      property: "kubernetesVersion",
      showAs: "text",
    },
    {
      displayName: "Cluster Name",
      property: "clusterName",
      showAs: "text",
    },
    {
      displayName: "Cluster Status",
      property: "clusterStatus",
      showAs: "text",
    },
    {
      displayName: "Nodegroup Name",
      property: "nodegroupName",
      showAs: "text",
    },
    {
      displayName: "Nodegroup Status",
      property: "nodegroupStatus",
      showAs: "text",
    },
    {
      displayName: "Endpoint",
      property: "endpoint",
      showAs: "url",
    },
    {
      displayName: "Certificate Authority",
      property: "certificateAuthority",
      showAs: "textarea",
    },
  ],
  hints: [
    {
      constraintName: "kubernetesVersion",
      hintValue: '"1.15" | "1.14" | "1.13" | "1.12"',
    },
  ],
  listEntityEventHeaders: [
    { text: "Name", value: "actionName" },
    { text: "By User", value: "userId" },
    { text: "Created At", value: "createTime" },
    { text: "Updated At", value: "updatedTime" },
    { text: "Finished", value: "finalized" },
    { text: "Success", value: "success" },
  ],
  listEntityEvents,
  getEntity,
  listEntities,
  pickComponent,
  streamEntityEvents,
  createEntity,
  icon: "mdi-ship-wheel",
});
