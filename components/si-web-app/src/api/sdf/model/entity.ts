import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";
import { Query, Comparison } from "@/api/sdf/model/query";
import {
  IListRequest,
  IListReply,
  IGetRequest,
  IGetReply,
} from "@/api/sdf/model";
import { sdf } from "@/api/sdf";
import _ from "lodash";
import store from "@/store";

export interface IEntity {
  id: string;
  name: string;
  description: string;
  objectType: string;
  expressionProperties: {
    [key: string]: Record<string, any>;
  };
  manualProperties: {
    [key: string]: Record<string, any>;
  };
  inferredProperties: {
    [key: string]: Record<string, any>;
  };
  properties: {
    [key: string]: Record<string, any>;
  };
  nodeId: string;
  head: boolean;
  siStorable: ISiStorable;
  siChangeSet: ISiChangeSet;
}

export class Entity implements IEntity {
  id: IEntity["id"];
  name: IEntity["name"];
  objectType: IEntity["objectType"];
  description: IEntity["description"];
  expressionProperties: IEntity["expressionProperties"];
  manualProperties: IEntity["manualProperties"];
  inferredProperties: IEntity["inferredProperties"];
  properties: IEntity["properties"];
  nodeId: IEntity["nodeId"];
  head: IEntity["head"];
  siStorable: IEntity["siStorable"];
  siChangeSet: IEntity["siChangeSet"];

  constructor(args: IEntity) {
    this.id = args.id;
    this.name = args.name;
    this.objectType = args.objectType;
    this.description = args.description;
    this.expressionProperties = args.expressionProperties;
    this.manualProperties = args.manualProperties;
    this.inferredProperties = args.inferredProperties;
    this.properties = args.properties;
    this.nodeId = args.nodeId;
    this.head = args.head;
    this.siStorable = args.siStorable;
    this.siChangeSet = args.siChangeSet;
  }

  static async get(request: IGetRequest<IEntity["id"]>): Promise<Entity> {
    const obj = await db.entities.get(request.id);
    if (obj) {
      return new Entity(obj);
    }
    const reply: IGetReply<IEntity> = await sdf.get(`entities/${request.id}`);
    const fetched: Entity = new Entity(reply.item);
    await fetched.save();
    return fetched;
  }

  static async list_by_object_type(
    objectType: string,
  ): Promise<IListReply<Entity>> {
    let items: Entity[] = [];
    let totalCount = 0;

    await db.entities
      .where("objectType")
      .equals(objectType)
      .each(obj => {
        items.push(new Entity(obj));
        totalCount = totalCount + 1;
      });

    if (totalCount == 0) {
      const result = await Entity.list({
        query: Query.for_simple_string(
          "objectType",
          "application",
          Comparison.Equals,
        ),
      });
      items = result.items;
      totalCount = result.totalCount;
    }

    return {
      items,
      totalCount,
    };
  }

  static async list(request?: IListRequest): Promise<IListReply<Entity>> {
    const items: Entity[] = [];
    let totalCount = 0;

    if (!request?.query) {
      await db.entities.each(obj => {
        items.push(new Entity(obj));
        totalCount = totalCount + 1;
      });
    }

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IEntity> = await sdf.list("entities", request);
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new Entity(item);
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
    const currentObj = await db.entities.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.entities.put(this);

      if (this.objectType == "application") {
        await store.dispatch("application/fromDb", this);
      }
    }
  }
}

db.entities.mapToClass(Entity);
