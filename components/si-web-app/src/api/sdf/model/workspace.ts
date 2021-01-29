import { sdf } from "@/api/sdf";
import { db } from "@/api/sdf/dexie";
import { ISimpleStorable } from "@/api/sdf/model/siStorable";
import {
  IGetRequest,
  IGetReply,
  IListRequest,
  IListReply,
} from "@/api/sdf/model";
import { Query, Comparison } from "@/api/sdf/model/query";
import _ from "lodash";
import Bottle from "bottlejs";

export interface IWorkspace {
  id: string;
  name: string;
  siStorable: ISimpleStorable;
}

export class Workspace implements IWorkspace {
  id: IWorkspace["id"];
  name: IWorkspace["name"];
  siStorable: IWorkspace["siStorable"];

  constructor(args: IWorkspace) {
    this.id = args.id;
    this.name = args.name;
    this.siStorable = args.siStorable;
  }

  static async get(request: IGetRequest<IWorkspace["id"]>): Promise<Workspace> {
    const obj = await db.workspaces.get(request.id);
    if (obj) {
      return new Workspace(obj);
    }
    const reply: IGetReply<IWorkspace> = await sdf.get(
      `workspaces/${request.id}`,
    );
    const fetched: Workspace = new Workspace(reply.item);
    fetched.save();
    return fetched;
  }

  static async find(index: "id" | "name", value: string): Promise<Workspace[]> {
    let items = await db.workspaces
      .where(index)
      .equals(value)
      .toArray();
    if (!items.length) {
      const results = await Workspace.list({
        query: Query.for_simple_string(index, value, Comparison.Equals),
      });
      return results.items;
    } else {
      return items.map(obj => new Workspace(obj));
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<Workspace>> {
    const items: Workspace[] = [];
    let totalCount = 0;

    await db.workspaces.each(obj => {
      items.push(new Workspace(obj));
      totalCount++;
    });

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IWorkspace> = await sdf.list(
          "workspaces",
          request,
        );
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new Workspace(item);
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
    const currentObj = await db.workspaces.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.workspaces.put(this);
      const bottle = Bottle.pop("default");
      const store = bottle.container.Store;
      await store.dispatch("workspace/fromDb", this);
    }
  }
}

db.workspaces.mapToClass(Workspace);
