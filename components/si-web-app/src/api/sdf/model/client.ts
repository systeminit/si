import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import _ from "lodash";
import store from "@/store";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ICreateReply, IGetRequest } from "@/api/sdf/model";
import sealedBox from "tweetnacl-sealedbox-js";

export enum ClientObjectType {
  Api = "api",
}

export enum ClientKind {
  Api = "api",
}

export enum ClientVersion {
  V1 = "v1",
}

export interface IClient {
  id: string;
  name: string;
  objectType: ClientObjectType;
  kind: ClientKind;
  siStorable: ISiStorable;
}

export interface IClientCreateRequest {
  name: string;
  objectType: ClientObjectType;
  kind: ClientKind;
  version: ClientVersion;
  organizationId: string;
  workspaceId: string;
}

export interface IClientCreate {
  name: string;
  objectType: ClientObjectType;
  kind: ClientKind;
  message: Uint8Array;
  organizationId: string;
  workspaceId: string;
}

export class Client implements IClient {
  id: IClient["id"];
  name: IClient["name"];
  objectType: IClient["objectType"];
  kind: IClient["kind"];
  siStorable: IClient["siStorable"];

  constructor(args: IClient) {
    this.id = args.id;
    this.name = args.name;
    this.objectType = args.objectType;
    this.kind = args.kind;
    this.siStorable = args.siStorable;
  }

  async save(): Promise<void> {
    const currentObj = await db.clients.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.clients.put(this);
      await this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("client/fromClient", this);
  }

  static async get(request: IGetRequest<IClient["id"]>): Promise<Client> {
    const obj = await db.clients.get(request.id);
    if (obj) {
      return new Client(obj);
    } else {
      throw new Error("client not found");
    }
  }

  static async restore(): Promise<void> {
    let iObjects = await db.clients.toArray();
    for (const iobj of iObjects) {
      let obj = new Client(iobj);
      await obj.dispatch();
    }
  }

  static async create(payload: IClientCreate): Promise<Client> {
    const request: IClientCreateRequest = {
      name: payload.name,
      objectType: payload.objectType,
      kind: payload.kind,
      version: ClientVersion.V1,
      organizationId: payload.organizationId,
      workspaceId: payload.workspaceId,
    };
    console.log("------> before")
    console.log(request)
    const reply: ICreateReply<IClient> = await sdf.post("clients", request);
    console.log("------> after")
    console.log(reply)
    const obj = new Client(reply.item);
    await obj.save();
    return obj;
  }

  static async findByObjectTypeAndKind(
    objectType: string,
    kind: string,
  ): Promise<Client[]> {
    let items = await db.clients.where({ objectType, kind }).toArray();
    return items.map(obj => new Client(obj));
  }
}

db.clients.mapToClass(Client);
