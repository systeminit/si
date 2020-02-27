import { SiComponent } from "@/registry/siComponent";

import getEntity from "./graphql/queries/getEntity.gql";
import listEntities from "./graphql/queries/listEntities.gql";
import pickComponent from "./graphql/queries/pickComponent.gql";
import listEntityEvents from "./graphql/queries/listEntityEvents.gql";

import createEntity from "./graphql/mutations/createEntity.gql";
import syncEntity from "./graphql/mutations/syncEntity.gql";

import streamEntityEvents from "./graphql/subscriptions/streamEntityEvents.gql";

export const sshKey = new SiComponent("sshKey", {
  name: "SSH Key",
  componentProperties: ["bits", "keyFormat", "keyType"],
  showActions: [
    { displayName: "Rotate" },
    { displayName: "Replace" },
    { displayName: "Clone" },
    { displayName: "Delete" },
    { displayName: "Sync", mutation: syncEntity },
  ],
  listHeaders: [
    { text: "Name", value: "name" },
    { text: "Key Type", value: "keyType" },
    { text: "Key Format", value: "keyFormat" },
    { text: "Bits", value: "bits" },
    { text: "State", value: "state" },
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
  showProperties: [
    { displayName: "ID", property: "id", showAs: "text" },
    { displayName: "Key Type", property: "keyType", showAs: "text" },
    { displayName: "Key Format", property: "keyFormat", showAs: "text" },
    { displayName: "Bits", property: "bits", showAs: "text" },
    { displayName: "Public Key", property: "publicKey", showAs: "textarea" },
    { displayName: "State", property: "state", showAs: "text" },
  ],
  hints: [
    { constraintName: "keyType", hintValue: '"RSA" | "DSA" | "ED25519"' },
    { constraintName: "keyFormat", hintValue: '"RFC4716" | "PKCS8" | "PEM"' },
    { constraintName: "bits", hintValue: "Number" },
  ],
  getEntity,
  listEntities,
  pickComponent,
  streamEntityEvents,
  createEntity,
  icon: "mdi-key",
});
