import {
  SiComponent,
  EntityAttrList,
  EntityPropSelect,
  EntityPropCode,
} from "@/registry/siComponent";

import getEntity from "./graphql/queries/getEntity.gql";
import listEntities from "./graphql/queries/listEntities.gql";
import pickComponent from "./graphql/queries/pickComponent.gql";
import streamEntityEvents from "./graphql/subscriptions/streamEntityEvents.gql";

import createEntity from "./graphql/mutations/createEntity.gql";
import syncEntity from "./graphql/mutations/syncEntity.gql";
import editEntity from "./graphql/mutations/editEntity.gql";
import editPropObjectYaml from "./graphql/mutations/editPropObjectYaml.gql";

import listEntityEvents from "./graphql/queries/listEntityEvents.gql";

export const kubernetesDeployment = new SiComponent("kubernetesDeployment", {
  name: "Kubernetes Deployment",
  componentProperties: ["kubernetesVersion"],
  listHeaders: [
    { text: "Name", value: "name" },
    { text: "Kubernetes Version", value: "kubernetesVersion" },
    { text: "State", value: "state" },
  ],
  showActions: [{ displayName: "Sync", mutation: syncEntity }],
  showProperties: [
    { displayName: "ID", property: "id", showAs: "text" },
    {
      displayName: "Kubernetes Version",
      property: "kubernetesVersion",
      showAs: "text",
    },
    {
      displayName: "Kubernetes Object",
      property: "object",
      showAs: "toml",
    },
    {
      displayName: "Kubernetes Yaml",
      property: "objectYaml",
      showAs: "textarea",
    },
  ],
  siSpec: {
    props: "object",
  },
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
  constraints: new EntityAttrList([
    new EntityPropSelect({
      name: "kubernetesVersion",
      label: "Kubernetes Version",
      required: true,
      defaultValue: "1.15",
      options: ["1.15", "1.14", "1.13", "1.12"],
    }),
  ]),
  properties: new EntityAttrList([
    new EntityPropCode({
      name: "object",
      label: "Kubernetes Object Spec",
      language: "toml",
      required: true,
      parsed: true,
    }),
  ]),

  listEntityEvents,
  getEntity,
  listEntities,
  pickComponent,
  streamEntityEvents,
  createEntity,
  editEntity,
  icon: "mdi-ship-wheel",
});
