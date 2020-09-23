import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import {
  IGetRequest,
  IGetReply,
  IListRequest,
  IListReply,
  ICreateReply,
} from "@/api/sdf/model";
import { Query, Comparison } from "@/api/sdf/model/query";
import store from "@/store";
import _ from "lodash";

export interface IEditSession {
  id: string;
  name: string;
  note: string;
  reverted: boolean;
  changeSetId: string;
  siStorable: ISiStorable;
}

export interface IEditSessionCreateRequest {
  name?: string;
  organizationId: string;
  workspaceId: string;
}

export class EditSession implements IEditSession {
  id: IEditSession["id"];
  name: IEditSession["name"];
  note: IEditSession["note"];
  reverted: IEditSession["reverted"];
  changeSetId: IEditSession["changeSetId"];
  siStorable: IEditSession["siStorable"];

  constructor(args: IEditSession) {
    this.id = args.id;
    this.name = args.name;
    this.note = args.note;
    this.reverted = args.reverted;
    this.changeSetId = args.changeSetId;
    this.siStorable = args.siStorable;
  }

  static async create(
    changeSetId: string,
    request: IEditSessionCreateRequest,
  ): Promise<EditSession> {
    const createReply: ICreateReply<IEditSession> = await sdf.post(
      `changeSets/${changeSetId}/editSessions`,
      request,
    );
    const obj = new EditSession(createReply.item);
    await obj.save();
    return obj;
  }

  static async get(
    request: IGetRequest<IEditSession["id"]>,
  ): Promise<EditSession> {
    const obj = await db.editSessions.get(request.id);
    if (obj) {
      return new EditSession(obj);
    }
    const reply: IGetReply<IEditSession> = await sdf.get(
      `editSessions/${request.id}`,
    );
    const fetched: EditSession = new EditSession(reply.item);
    fetched.save();
    return fetched;
  }

  static async find(
    index: "id" | "name",
    value: string,
  ): Promise<EditSession[]> {
    let items = await db.editSessions
      .where(index)
      .equals(value)
      .toArray();
    if (!items.length) {
      const results = await EditSession.list({
        query: Query.for_simple_string(index, value, Comparison.Equals),
      });
      return results.items;
    } else {
      return items.map(obj => new EditSession(obj));
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<EditSession>> {
    const items: EditSession[] = [];
    let totalCount = 0;

    db.editSessions.each(obj => {
      items.push(new EditSession(obj));
      totalCount++;
    });

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IEditSession> = await sdf.list(
          "editSessions",
          request,
        );
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new EditSession(item);
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
    const currentObj = await db.editSessions.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.editSessions.put(this);
      await store.dispatch("editSessions/fromDb", this);
    }
  }
}

db.editSessions.mapToClass(EditSession);
