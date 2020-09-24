import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";
import {
  IListRequest,
  IListReply,
  IGetRequest,
  IGetReply,
} from "@/api/sdf/model";
import { Query, Comparison } from "@/api/sdf/model/query";
import { sdf } from "@/api/sdf";
import _ from "lodash";
import store from "@/store";

export interface ISystem {
  id: string;
  name: string;
  description: string;
  nodeId: string;
  head: boolean;
  siStorable: ISiStorable;
  siChangeSet: ISiChangeSet;
}

export class System implements ISystem {
  id: ISystem["id"];
  name: ISystem["name"];
  description: ISystem["description"];
  nodeId: ISystem["nodeId"];
  head: ISystem["head"];
  siStorable: ISystem["siStorable"];
  siChangeSet: ISystem["siChangeSet"];

  constructor(args: ISystem) {
    this.id = args.id;
    this.name = args.name;
    this.description = args.description;
    this.nodeId = args.nodeId;
    this.head = args.head;
    this.siStorable = args.siStorable;
    this.siChangeSet = args.siChangeSet;
  }

  static async get(request: IGetRequest<ISystem["id"]>): Promise<System> {
    const obj = await db.systems.get(request.id);
    if (obj) {
      return new System(obj);
    }
    const reply: IGetReply<ISystem> = await sdf.get(`systems/${request.id}`);
    const fetched: System = new System(reply.item);
    await fetched.save();
    return fetched;
  }

  static async find(index: "id" | "name", value: string): Promise<System[]> {
    let items = await db.systems
      .where(index)
      .equals(value)
      .toArray();
    if (!items.length) {
      const results = await System.list({
        query: Query.for_simple_string(index, value, Comparison.Equals),
      });
      return results.items;
    } else {
      return items.map(obj => new System(obj));
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<System>> {
    const items: System[] = [];
    let totalCount = 0;

    if (!request?.query) {
      await db.entities.each(obj => {
        items.push(new System(obj));
        totalCount = totalCount + 1;
      });
    }

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<ISystem> = await sdf.list("systems", request);
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new System(item);
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
    const currentObj = await db.systems.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.systems.put(this);
      await store.dispatch("system/fromDb", this);
    }
  }
}

db.systems.mapToClass(System);
