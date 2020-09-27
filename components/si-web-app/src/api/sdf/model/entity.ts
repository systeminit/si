import { db } from "@/api/sdf/dexie";
import { ChangeSetStatus } from "@/api/sdf/model/changeSet";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { ISiChangeSet } from "@/api/sdf/model/siChangeSet";
import { Edge, EdgeKind } from "@/api/sdf/model/edge";
import {
  Query,
  Comparison,
  BooleanTerm,
  FieldType,
} from "@/api/sdf/model/query";
import { System } from "@/api/sdf/model/system";
import { IListRequest, IListReply, IGetRequest } from "@/api/sdf/model";
import { sdf } from "@/api/sdf";
import _ from "lodash";
import store from "@/store";

export interface IEntityGetReply {
  items: IEntity[];
}

export interface IEntityGetProjectionRequest {
  id: string;
  changeSetId: string;
}

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

  static upgrade(obj: Entity | IEntity): Entity {
    if (obj instanceof Entity) {
      return obj;
    } else {
      return new Entity(obj);
    }
  }

  static async get_any(request: IGetRequest<IEntity["id"]>): Promise<Entity> {
    let entity;
    try {
      entity = await Entity.get_head(request);
    } catch (err) {
      let iEntity = await db.projectionEntities
        .where({ id: request.id })
        .first();
      if (iEntity) {
        entity = new Entity(iEntity);
      }
    }
    if (entity) {
      return entity;
    } else {
      throw new Error("cannot find any entity");
    }
  }

  static async get_head(request: IGetRequest<IEntity["id"]>): Promise<Entity> {
    const obj = await db.headEntities.get(request.id);
    if (obj) {
      return new Entity(obj);
    }
    const reply: IEntityGetReply = await sdf.get(`entities/${request.id}`);
    let head_entity: Entity | undefined;
    for (let ientity of reply.items) {
      let entity = new Entity(ientity);
      if (entity.head) {
        head_entity = entity;
      }
      entity.save();
    }
    if (head_entity) {
      return head_entity;
    } else {
      throw new Error("cannot find head entity");
    }
  }

  static async get_projection(
    request: IEntityGetProjectionRequest,
  ): Promise<Entity> {
    const obj = await db.projectionEntities.get({
      id: request.id,
      "siChangeSet.changeSetId": request.changeSetId,
    });
    if (obj) {
      return new Entity(obj);
    }
    const reply: IEntityGetReply = await sdf.get(`entities/${request.id}`);
    let projection_entity: Entity | undefined;
    for (let ientity of reply.items) {
      let entity = new Entity(ientity);
      if (entity.siChangeSet.changeSetId == request.changeSetId) {
        projection_entity = entity;
      }
      entity.save();
    }
    if (projection_entity) {
      return projection_entity;
    } else {
      throw new Error("cannot find projection entity");
    }
  }

  static async list_head_by_object_type(
    objectType: string,
  ): Promise<IListReply<Entity>> {
    let items: Entity[] = [];
    let totalCount = 0;

    await db.headEntities
      .where("objectType")
      .equals(objectType)
      .each(obj => {
        items.push(new Entity(obj));
        totalCount = totalCount + 1;
      });

    if (totalCount == 0) {
      const result = await Entity.list_head({
        query: {
          booleanTerm: BooleanTerm.And,
          items: [
            {
              expression: {
                field: "objectType",
                value: "application",
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
            {
              expression: {
                field: "head",
                value: "true",
                comparison: Comparison.Equals,
                fieldType: FieldType.Boolean,
              },
            },
          ],
        },
      });
      items = result.items;
      totalCount = result.totalCount;
    }

    return {
      items,
      totalCount,
    };
  }

  static async list_head(request?: IListRequest): Promise<IListReply<Entity>> {
    const items: Entity[] = [];
    let totalCount = 0;

    if (!request?.query) {
      await db.headEntities.each(obj => {
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

  async changeSetCounts(): Promise<{ open: number; closed: number }> {
    const reply = { open: 0, closed: 0 };

    const successors = await this.successors();
    console.log("changeSetCount successors", { entity: this, successors });
    const objectIds = _.map(successors, e => e.id);
    console.log("changeSetCount objectIds", { entity: this, objectIds });
    objectIds.push(this.id);
    let changeSets: Set<string> = new Set([]);
    await db.changeSetParticipants
      .where("objectId")
      .anyOf(objectIds)
      .each(csp => {
        changeSets.add(csp.changeSetId);
      });
    console.log("changeSetCount changeSets", { entity: this, changeSets });
    await db.changeSets
      .where("id")
      .anyOf(Array.from(changeSets))
      .each(changeSet => {
        if (changeSet.status == ChangeSetStatus.Open) {
          reply.open++;
        } else if (changeSet.status == ChangeSetStatus.Closed) {
          reply.closed++;
        }
      });
    console.log("changeSetCount reply", { entity: this, reply });

    return reply;
  }

  // Returns the entities that are successors to this entity in the configuration graph
  async successors(): Promise<Entity[]> {
    let edges = await Edge.allSuccessors({
      objectId: this.id,
      edgeKind: EdgeKind.Configures,
    });
    let items: Entity[] = [];
    for (let edge of edges) {
      let entity = await Entity.get_any({ id: edge.headVertex.objectId });
      items.push(entity);
    }
    return items;
  }

  async systems(): Promise<System[]> {
    let edges = await Edge.byTailTypeForHeadObjectId(
      EdgeKind.Includes,
      "system",
      this.id,
    );
    let items: System[] = [];
    for (let edge of edges) {
      let system = await System.get({ id: edge.tailVertex.objectId });
      items.push(system);
    }
    return items;
  }

  async save(): Promise<void> {
    if (this.head) {
      await db.headEntities.put(this);
      await store.dispatch("application/fromEntity", this);
    } else {
      await db.projectionEntities.put(this);
    }
  }
}

db.headEntities.mapToClass(Entity);
db.projectionEntities.mapToClass(Entity);
