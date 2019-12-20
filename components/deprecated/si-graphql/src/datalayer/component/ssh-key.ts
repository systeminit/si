import { CreateComponent, ComponentObject } from "@/datalayer/component";
import { CreateEntity, EntityObject } from "@/datalayer/entity";

export interface SshKeyComponent extends ComponentObject {
  keyType: "DSA" | "ECDSA" | "ED25519" | "RSA";
  keyFormat: "RFC4716" | "PKCS8" | "PEM";
  bits: number;
}

export interface SshKeyEntity extends EntityObject {
  keyType: SshKeyComponent["keyType"];
  keyFormat: SshKeyComponent["keyFormat"];
  bits: SshKeyComponent["bits"];
  comment: string;
  bubbleBabble: string;
  fingerPrint: string;
  randomArt: string;
  privateKey: string;
  publicKey: string;
}

export const SshKey = CreateComponent<SshKeyComponent>({
  __typename: "SshKeyComponent",
  nodeType: "SSH Key",
  fqKey: "component:sshkey",
});

export const SshKeyEntity = CreateEntity<SshKeyEntity>({
  __typename: "SshKeyEntity",
  nodeType: "SSH Key",
  fqKey: "entity:sshkey",
});
