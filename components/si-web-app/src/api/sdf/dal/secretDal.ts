import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import _ from "lodash";
import {
  Secret,
  SecretAlgorithm,
  SecretKind,
  SecretObjectType,
  SecretVersion,
  ISecret,
} from "@/api/sdf/model/secret";
import { PublicKey } from "@/api/sdf/model/publicKey";

export interface IGetPublicKeyReplySuccess {
  publicKey: PublicKey;
  error?: never;
}

export interface IGetPublicKeyReplyFailure {
  publicKey?: never;
  error: SDFError;
}

export type IGetPublicKeyReply =
  | IGetPublicKeyReplySuccess
  | IGetPublicKeyReplyFailure;

async function getPublicKey(): Promise<IGetPublicKeyReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: IGetPublicKeyReply = await sdf.get("secretDal/getPublicKey");
  if (!reply.error) {
    reply.publicKey = PublicKey.upgrade(reply.publicKey);
  }
  return reply;
}

export interface ICreateSecretRequest {
  name: string;
  objectType: SecretObjectType;
  kind: SecretKind;
  crypted: number[];
  keyPairId: string;
  version: SecretVersion;
  algorithm: SecretAlgorithm;
  workspaceId: string;
}

export interface ICreateSecretReplySuccess {
  secret: Secret;
  error?: never;
}

export interface ICreateSecretReplyFailure {
  secret?: never;
  error: SDFError;
}

export type ICreateSecretReply =
  | ICreateSecretReplySuccess
  | ICreateSecretReplyFailure;

async function createSecret(
  request: ICreateSecretRequest,
): Promise<ICreateSecretReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: ICreateSecretReply = await sdf.post(
    "secretDal/createSecret",
    request,
  );
  return reply;
}

export interface IListSecretsRequest {
  workspaceId: string;
}

export interface IListSecretsReplySuccess {
  list: ISecret[];
  error?: never;
}

export interface IListSecretsReplyFailure {
  list?: never;
  error: SDFError;
}

export type IListSecretsReply =
  | IListSecretsReplySuccess
  | IListSecretsReplyFailure;

async function listSecrets(
  request: IListSecretsRequest,
): Promise<IListSecretsReply> {
  const bottle = Bottle.pop("default");
  const sdf = bottle.container.SDF;

  const reply: IListSecretsReply = await sdf.get(
    "secretDal/listSecretsForWorkspace",
    request,
  );
  return reply;
}

export const SecretDal = {
  getPublicKey,
  createSecret,
  listSecrets,
};
