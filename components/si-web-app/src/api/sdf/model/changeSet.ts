import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import {
  IGetRequest,
  IGetReply,
  IListRequest,
  IListReply,
} from "@/api/sdf/model";
import { Query, Comparison } from "@/api/sdf/model/query";
import store from "@/store";
import _ from "lodash";

export enum ChangeSetStatus {
  Open = "open",
  Closed = "closed",
  Abandoned = "abandoned",
  Executing = "executing",
  Failed = "failed",
}

export interface IChangeSet {
  id: string;
  name: string;
  note: string;
  status: ChangeSetStatus;
  siStorable: ISiStorable;
}

export interface IChangeSetCreateRequest {
  name?: string;
  organizationId: string;
  workspaceId: string;
}

export interface IChangeSetCreateReply {
  item: IChangeSet;
}

export interface IChangeSetExecuteRequest {
  hypothetical: boolean;
}

export interface IChangeSetPatchOps {
  execute?: IChangeSetExecuteRequest;
}

export interface IChangeSetPatchRequest {
  op: IChangeSetPatchOps;
  organizationId: string;
  workspaceId: string;
}

export interface IChangeSetPatchReply {
  execute?: string[];
}

export class ChangeSet implements IChangeSet {
  id: IChangeSet["id"];
  name: IChangeSet["name"];
  note: IChangeSet["note"];
  status: IChangeSet["status"];
  siStorable: IChangeSet["siStorable"];

  constructor(args: IChangeSet) {
    this.id = args.id;
    this.name = args.name;
    this.note = args.note;
    this.status = args.status;
    this.siStorable = args.siStorable;
  }

  static async create(request: IChangeSetCreateRequest): Promise<ChangeSet> {
    const createReply: IChangeSetCreateReply = await sdf.post(
      "changeSets",
      request,
    );
    const obj = new ChangeSet(createReply.item);
    await obj.save();
    return obj;
  }

  static async get(request: IGetRequest<IChangeSet["id"]>): Promise<ChangeSet> {
    const obj = await db.changeSets.get(request.id);
    if (obj) {
      return new ChangeSet(obj);
    }
    const reply: IGetReply<IChangeSet> = await sdf.get(
      `changeSets/${request.id}`,
    );
    const fetched: ChangeSet = new ChangeSet(reply.item);
    fetched.save();
    return fetched;
  }

  static async find(index: "id" | "name", value: string): Promise<ChangeSet[]> {
    let items = await db.changeSets
      .where(index)
      .equals(value)
      .toArray();
    if (!items.length) {
      const results = await ChangeSet.list({
        query: Query.for_simple_string(index, value, Comparison.Equals),
      });
      return results.items;
    } else {
      return items.map(obj => new ChangeSet(obj));
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<ChangeSet>> {
    const items: ChangeSet[] = [];
    let totalCount = 0;

    db.changeSets.each(obj => {
      items.push(new ChangeSet(obj));
      totalCount++;
    });

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IChangeSet> = await sdf.list(
          "changeSets",
          request,
        );
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new ChangeSet(item);
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

  async execute(request: IChangeSetExecuteRequest): Promise<void> {
    let full_request: IChangeSetPatchRequest = {
      op: {
        execute: request,
      },
      workspaceId: this.siStorable.workspaceId,
      organizationId: this.siStorable.organizationId,
    };
    await sdf.patch(`changeSets/${this.id}`, full_request);
  }

  async save(): Promise<void> {
    const currentObj = await db.changeSets.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.changeSets.put(this);
      await store.dispatch("application/fromDb", this);
    }
  }
}

db.changeSets.mapToClass(ChangeSet);
