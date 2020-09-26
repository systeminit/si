import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISiStorable, ISimpleStorable } from "@/api/sdf/model/siStorable";
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
      await store.dispatch("changeSet/fromChangeSet", this);
      await store.dispatch("application/fromChangeSet", this);
    }
  }
}

db.changeSets.mapToClass(ChangeSet);

export interface IChangeSetParticipant {
  id: string;
  changeSetId: string;
  objectId: string;
  siStorable: ISimpleStorable;
}

export class ChangeSetParticipant implements IChangeSetParticipant {
  id: IChangeSetParticipant["id"];
  changeSetId: IChangeSetParticipant["changeSetId"];
  objectId: IChangeSetParticipant["objectId"];
  siStorable: IChangeSetParticipant["siStorable"];

  constructor(args: IChangeSetParticipant) {
    this.id = args.id;
    this.changeSetId = args.changeSetId;
    this.objectId = args.objectId;
    this.siStorable = args.siStorable;
  }

  // To get the list of changesets that apply to a given application, we need to know
  // the full set of descendent edges for the application, and then we need to know all
  // the changeSets those edges participate in.
  //
  // We really want to make a request as simple as entity/changeSets - and then have it
  // find all the right ones!

  static async list(
    request?: IListRequest,
  ): Promise<IListReply<ChangeSetParticipant>> {
    const items: ChangeSetParticipant[] = [];
    let totalCount = 0;

    if (!request?.query) {
      await db.changeSetParticipants.each(obj => {
        items.push(new ChangeSetParticipant(obj));
        totalCount = totalCount + 1;
      });
    }

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IChangeSetParticipant> = await sdf.list(
          "changeSetParticipants",
          request,
        );
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new ChangeSetParticipant(item);
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
    const currentObj = await db.changeSetParticipants.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.changeSetParticipants.put(this);
      await store.dispatch("application/fromChangeSetParticipant", this);
    }
  }
}

db.changeSetParticipants.mapToClass(ChangeSetParticipant);
