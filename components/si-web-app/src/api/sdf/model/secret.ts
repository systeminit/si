import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import _ from "lodash";
import store from "@/store";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ICreateReply, IGetRequest } from "@/api/sdf/model";
import { PublicKey } from "./keyPair";
import sealedBox from "tweetnacl-sealedbox-js";

export enum SecretObjectType {
  Credential = "credential",
}

export enum SecretKind {
  DockerHub = "dockerHub",
}

export enum SecretVersion {
  V1 = "v1",
}

export enum SecretAlgorithm {
  Sealedbox = "sealedbox",
}

export interface ISecret {
  id: string;
  name: string;
  objectType: SecretObjectType;
  kind: SecretKind;
  siStorable: ISiStorable;
}

export interface ISecretCreateRequest {
  name: string;
  objectType: SecretObjectType;
  kind: SecretKind;
  crypted: number[];
  keyPairId: string;
  version: SecretVersion;
  algorithm: SecretAlgorithm;
  organizationId: string;
  workspaceId: string;
}

export interface ISecretCreate {
  name: string;
  objectType: SecretObjectType;
  kind: SecretKind;
  message: Uint8Array;
  publicKey: PublicKey;
  organizationId: string;
  workspaceId: string;
}

export class Secret implements ISecret {
  id: ISecret["id"];
  name: ISecret["name"];
  objectType: ISecret["objectType"];
  kind: ISecret["kind"];
  siStorable: ISecret["siStorable"];

  constructor(args: ISecret) {
    this.id = args.id;
    this.name = args.name;
    this.objectType = args.objectType;
    this.kind = args.kind;
    this.siStorable = args.siStorable;
  }

  async save(): Promise<void> {
    const currentObj = await db.secrets.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.secrets.put(this);
      await this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("secret/fromSecret", this);
  }

  static async get(request: IGetRequest<ISecret["id"]>): Promise<Secret> {
    const obj = await db.secrets.get(request.id);
    if (obj) {
      return new Secret(obj);
    } else {
      throw new Error("secret not found");
    }
  }

  static async restore(): Promise<void> {
    let iObjects = await db.secrets.toArray();
    for (const iobj of iObjects) {
      let obj = new Secret(iobj);
      await obj.dispatch();
    }
  }

  static async create(payload: ISecretCreate): Promise<Secret> {
    const crypted = sealedBox.seal(
      payload.message,
      payload.publicKey.publicKey,
    );
    const request: ISecretCreateRequest = {
      name: payload.name,
      objectType: payload.objectType,
      kind: payload.kind,
      crypted: Array.from(crypted),
      keyPairId: payload.publicKey.id,
      version: SecretVersion.V1,
      algorithm: SecretAlgorithm.Sealedbox,
      organizationId: payload.organizationId,
      workspaceId: payload.workspaceId,
    };
    const reply: ICreateReply<ISecret> = await sdf.post("secrets", request);
    const obj = new Secret(reply.item);
    await obj.save();
    return obj;
  }

  static async findByObjectTypeAndKind(
    objectType: string,
    kind: string,
  ): Promise<Secret[]> {
    let items = await db.secrets.where({ objectType, kind }).toArray();
    return items.map(obj => new Secret(obj));
  }
}

db.secrets.mapToClass(Secret);
