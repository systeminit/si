import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
import {
  IGetRequest,
  IGetReply,
  ICreateReply,
  IListRequest,
  IListReply,
} from "@/api/sdf/model";
import store from "@/store";
import _ from "lodash";

export interface IApiClientCreateRequest {
  name: string;
  kind: ApiClientKind;
}

export interface IApiClientCreateReply {
  apiClient: IApiClient;
  token: string;
}

export interface ApiClientCreateReply {
  apiClient: ApiClient;
  token: string;
}

export enum ApiClientKind {
  Cli = "cli",
}

export interface IApiClient {
  id: string;
  name: string;
  validTokenHash: string;
  kind: ApiClientKind;
  siStorable: ISimpleStorable;
}

export class ApiClient implements IApiClient {
  id: IApiClient["id"];
  name: IApiClient["name"];
  kind: ApiClientKind;
  validTokenHash: IApiClient["validTokenHash"];
  siStorable: IApiClient["siStorable"];

  constructor(args: IApiClient) {
    this.id = args.id;
    this.name = args.name;
    this.kind = args.kind;
    this.validTokenHash = args.validTokenHash;
    this.siStorable = args.siStorable;
  }

  static async create(
    request: IApiClientCreateRequest,
  ): Promise<ApiClientCreateReply> {
    const reply: IApiClientCreateReply = await sdf.post("apiClients", request);
    const obj = new ApiClient(reply.apiClient);
    await obj.save();
    return { apiClient: obj, token: reply.token };
  }

  static async list(request?: IListRequest): Promise<IListReply<ApiClient>> {
    const items: ApiClient[] = [];
    let totalCount = 0;

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IApiClient> = await sdf.list(
          "apiClients",
          request,
        );
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new ApiClient(item);
            objItem.save();
            items.push(objItem);
          }
        }
        if (reply.pageToken) {
          request = {
            pageToken: reply.pageToken,
          };
        } else {
          totalCount = reply.totalCount;
          finished = true;
        }
      }
    }
    return {
      items,
      totalCount,
    };
  }

  async save(): Promise<void> {
    const currentObj = await db.apiClients.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.apiClients.put(this);
      await this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("client/fromApiClient", this);
  }
}

db.apiClients.mapToClass(ApiClient);
