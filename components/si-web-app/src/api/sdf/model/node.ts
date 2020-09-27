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
import { IEdge, Edge } from "@/api/sdf/model/edge";
import { Query, Comparison } from "@/api/sdf/model/query";
import { Entity, IEntity } from "@/api/sdf/model/entity";
import { System, ISystem } from "@/api/sdf/model/system";
import store from "@/store";
import _ from "lodash";

export type INodeObject = IEntity | ISystem;
export type NodeObject = Entity | System;

export enum NodeKind {
  Entity = "Entity",
  System = "System",
}

export interface Position {
  x: number;
  y: number;
}

export interface INode {
  id: string;
  positions: {
    [key: string]: Position;
  };
  kind: NodeKind;
  objectType: string;
  siStorable: ISiStorable;
}

export interface INodeCreateRequest {
  name?: string;
  kind: NodeKind;
  objectType: string;
  organizationId: string;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
  systemIds?: string[];
}

export interface INodePatchIncludeSystemReply {
  includeSystem: {
    edge: IEdge;
  };
}

export interface INodePatchConfiguredByReply {
  configuredBy: {
    edge: IEdge;
  };
}

export interface INodePatchOpRequest {
  includeSystem?: {
    systemId: string;
  };
  configuredBy?: {
    nodeId: string;
  };
}

export interface INodePatchRequest {
  op: INodePatchOpRequest;
  organizationId: string;
  workspaceId: string;
}

export interface INodePatchReply {
  includeSystem?: {
    edge: IEdge;
  };
  configuredBy?: {
    edge: IEdge;
  };
}

export class Node implements INode {
  id: INode["id"];
  positions: INode["positions"];
  kind: INode["kind"];
  objectType: INode["objectType"];
  siStorable: INode["siStorable"];

  constructor(args: INode) {
    this.id = args.id;
    this.positions = args.positions;
    this.kind = args.kind;
    this.objectType = args.objectType;
    this.siStorable = args.siStorable;
  }

  static async create(request: INodeCreateRequest): Promise<Node> {
    const reply: ICreateReply<INode> = await sdf.post("nodes", request);
    const obj = new Node(reply.item);
    await obj.save();
    return obj;
  }

  static async get(request: IGetRequest<INode["id"]>): Promise<Node> {
    const obj = await db.nodes.get(request.id);
    if (obj) {
      return new Node(obj);
    }
    const reply: IGetReply<INode> = await sdf.get(`nodes/${request.id}`);
    const fetched: Node = new Node(reply.item);
    fetched.save();
    return fetched;
  }

  static async find(index: "id", value: string): Promise<Node[]> {
    let items = await db.nodes
      .where(index)
      .equals(value)
      .toArray();
    if (!items.length) {
      const results = await Node.list({
        query: Query.for_simple_string(index, value, Comparison.Equals),
      });
      return results.items;
    } else {
      return items.map(obj => new Node(obj));
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<Node>> {
    const items: Node[] = [];
    let totalCount = 0;

    db.nodes.each(obj => {
      items.push(new Node(obj));
      totalCount++;
    });

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<INode> = await sdf.list("nodes", request);
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new Node(item);
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

  async configured_by(nodeId: string): Promise<Edge> {
    let request: INodePatchRequest = {
      op: { configuredBy: { nodeId } },
      organizationId: this.siStorable.organizationId,
      workspaceId: this.siStorable.workspaceId,
    };
    let reply: INodePatchReply = await sdf.patch(`nodes/${this.id}`, request);
    if (reply.configuredBy) {
      let edge = new Edge(reply.configuredBy.edge);
      await edge.save();
      return edge;
    } else {
      throw new Error("incorrect response to patch call");
    }
  }

  async include_in_system(systemId: string): Promise<Edge> {
    let request: INodePatchRequest = {
      op: { includeSystem: { systemId } },
      organizationId: this.siStorable.organizationId,
      workspaceId: this.siStorable.workspaceId,
    };
    let reply: INodePatchReply = await sdf.patch(`nodes/${this.id}`, request);
    if (reply.includeSystem) {
      let edge = new Edge(reply.includeSystem.edge);
      await edge.save();
      return edge;
    } else {
      throw new Error("incorrect response to patch call");
    }
  }

  async displayObject(changeSetId?: string): Promise<NodeObject> {
    console.log("chekcing on the display", { changeSetId, node: this });
    let displayObject;
    try {
      if (changeSetId) {
        displayObject = await this.projectionObject(changeSetId);
        return displayObject;
      }
    } catch {}
    if (!displayObject) {
      displayObject = await this.headObject();
      return displayObject;
    }
    throw new Error("cannot get display object; no head or projection");
  }

  async headObject(): Promise<NodeObject> {
    let iitem: INodeObject;
    let cacheResult = await db.headEntities
      .where({ nodeId: this.id })
      .toArray();
    if (cacheResult.length && cacheResult[0]) {
      iitem = cacheResult[0];
    } else {
      let response: IGetReply<INodeObject> = await sdf.get(
        `nodes/${this.id}/object`,
      );
      iitem = response.item;
    }
    if (iitem.siStorable.typeName == "system") {
      return new System(iitem as ISystem);
    } else if (iitem.siStorable.typeName == "entity") {
      return new Entity(iitem as IEntity);
    } else {
      throw new Error("unknown object type");
    }
  }

  async projectionObject(changeSetId: string): Promise<NodeObject> {
    let iitem: INodeObject;
    let cacheResult = await db.projectionEntities
      .where({ nodeId: this.id, "siChangeSet.changeSetId": changeSetId })
      .toArray();
    if (cacheResult.length && cacheResult[0]) {
      iitem = cacheResult[0];
    } else {
      let response: IGetReply<INodeObject> = await sdf.get(
        `nodes/${this.id}/object`,
        { changeSetId },
      );
      iitem = response.item;
    }
    if (iitem.siStorable.typeName == "system") {
      return new System(iitem as ISystem);
    } else if (iitem.siStorable.typeName == "entity") {
      return new Entity(iitem as IEntity);
    } else {
      throw new Error("unknown object type");
    }
  }

  async save(): Promise<void> {
    const currentObj = await db.nodes.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.nodes.put(this);
      await store.dispatch("editor/fromNode", this);
    }
  }
}

db.nodes.mapToClass(Node);
