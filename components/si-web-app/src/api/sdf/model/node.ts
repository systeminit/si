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
import { IEdge, Edge, EdgeKind } from "@/api/sdf/model/edge";
import { Query, Comparison } from "@/api/sdf/model/query";
import { Entity, IEntity } from "@/api/sdf/model/entity";
import { System, ISystem } from "@/api/sdf/model/system";
import store from "@/store";

import _ from "lodash";
import { registry, Props, PropMethod, PropLink, PropObject } from "si-registry";

export interface RegistryProperty {
  id: string;
  path: string[];
  prop: Props;
  name: string;
  label: string;
  required: boolean;
  repeated: boolean;
  kind: string;
  hidden: boolean;
}

export interface PropEntry {
  prop: Props;
  path: string[];
}

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
  setPosition?: {
    context: string;
    position: Position;
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
  setPosition?: {
    context: string;
    position: Position;
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

  static upgrade(obj: Node | INode): Node {
    if (obj instanceof Node) {
      return obj;
    } else {
      return new Node(obj);
    }
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

  async configuredBy(nodeId: string): Promise<Edge> {
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

  async includeInSystem(systemId: string): Promise<Edge> {
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

  async predecessors(): Promise<Node[]> {
    let edges = await Edge.allPredecessors({
      nodeId: this.id,
      edgeKind: EdgeKind.Configures,
    });
    let items: Node[] = [];
    for (let edge of edges) {
      let node = await Node.get({ id: edge.tailVertex.nodeId });
      items.push(node);
    }
    return items;
  }

  async successors(): Promise<Node[]> {
    let edges = await Edge.allSuccessors({
      nodeId: this.id,
      edgeKind: EdgeKind.Configures,
    });
    let items: Node[] = [];
    for (let edge of edges) {
      let node = await Node.get({ id: edge.headVertex.nodeId });
      items.push(node);
    }
    return items;
  }

  async successorEdges(): Promise<Edge[]> {
    let edges = await Edge.allSuccessors({
      nodeId: this.id,
      edgeKind: EdgeKind.Configures,
    });
    return edges;
  }

  async displayObject(changeSetId?: string): Promise<NodeObject> {
    let displayObject;
    try {
      if (changeSetId) {
        displayObject = await this.projectionObject(changeSetId);
        return displayObject;
      }
    } catch {
      console.log("failed to find projection");
    }
    if (!displayObject) {
      try {
        displayObject = await this.headObject();
        return displayObject;
      } catch {
        console.log("failed to find head object");
      }
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
    console.log("looking for projection", {
      nodeId: this.id,
      changeSetId: changeSetId,
    });
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

  async setPosition(context: string, position: Position) {
    this.positions[context] = position;
    let request: INodePatchRequest = {
      op: { setPosition: { context, position } },
      organizationId: this.siStorable.organizationId,
      workspaceId: this.siStorable.workspaceId,
    };
    await sdf.patch(`nodes/${this.id}`, request);
    this.save();
  }

  position(context: string): Position {
    if (this.positions[context]) {
      return this.positions[context];
    } else {
      this.positions[context] = {
        x: 0,
        y: 0,
      };
      return this.positions[context];
    }
  }

  async propertyList(changeSetId?: string): Promise<RegistryProperty[]> {
    let entity = await this.displayObject(changeSetId);
    let registryObject;
    if (entity.siStorable.typeName == "entity") {
      // @ts-ignore
      registryObject = registry.get(entity.objectType);
    } else {
      registryObject = registry.get("system");
    }

    const properties = registryObject.fields.getEntry(
      "properties",
    ) as PropObject;
    const objectProperties: PropEntry[] = properties.properties.attrs.map(
      prop => {
        return { prop, path: [] };
      },
    );
    const result: RegistryProperty[] = [];

    for (const propEntry of objectProperties) {
      let path = propEntry.path;
      let prop = propEntry.prop;
      path.push(prop.name);

      if (prop.kind() == "link") {
        let cprop = prop as PropLink;
        const realProp = cprop.lookupMyself();

        result.push({
          id: `${this.id}-${path.join("-")}-${changeSetId}-${
            this.siStorable.updateClock.epoch
          }-${this.siStorable.updateClock.updateCount}`,
          name: prop.name,
          label: prop.label,
          path,
          prop: realProp,
          required: prop.required,
          repeated: prop.repeated,
          kind: realProp.kind(),
          hidden: prop.hidden,
        });
        if (realProp.kind() == "object" && prop.repeated == false) {
          const rProp = realProp as PropObject;
          let newProps = rProp.properties.attrs.map(prop => {
            return { prop, path: _.clone(path) };
          });
          for (let nProp of newProps) {
            objectProperties.push(nProp);
          }
        }
      } else {
        if (prop.kind() == "object" && prop.repeated == false) {
          const rProp = prop as PropObject;
          let newProps = rProp.properties.attrs.map(prop => {
            return { prop, path: _.clone(path) };
          });
          for (let nProp of newProps) {
            objectProperties.push(nProp);
          }
        }
        result.push({
          id: `${this.id}-${path.join("-")}-${changeSetId}-${
            this.siStorable.updateClock.epoch
          }-${this.siStorable.updateClock.updateCount}`,
          name: prop.name,
          label: prop.label,
          path,
          prop,
          required: prop.required,
          repeated: prop.repeated,
          kind: prop.kind(),
          hidden: prop.hidden,
        });
      }
    }
    // This groups things according to their nesting, so we can just
    // walk the results and have everything in the proper order.
    const grouped = _.groupBy(result, value => {
      if (value.kind == "object") {
        return value.path;
      } else {
        return value.path.slice(0, -1);
      }
    });
    return _.flatten(Object.values(grouped));
  }

  async propertyListRepeated(
    entityProperty: RegistryProperty,
    index: number,
    changeSetId?: string,
  ): Promise<RegistryProperty[]> {
    if (entityProperty.kind == "object") {
      let updateField = entityProperty.prop as PropObject;

      const objectProperties: PropEntry[] = updateField.properties.attrs.map(
        prop => {
          return { prop, path: _.clone(entityProperty.path) };
        },
      );
      const result: RegistryProperty[] = [];

      for (const propEntry of objectProperties) {
        let path = propEntry.path;
        let prop = propEntry.prop;
        path.push(`${index}`);
        path.push(prop.name);

        if (prop.kind() == "link") {
          let cprop = prop as PropLink;
          const realProp = cprop.lookupMyself();

          result.push({
            id: `${this.id}-${path.join("-")}-${changeSetId}-${
              this.siStorable.updateClock.epoch
            }-${this.siStorable.updateClock.updateCount}`,
            name: prop.name,
            label: prop.label,
            path,
            prop: realProp,
            required: prop.required,
            repeated: prop.repeated,
            kind: realProp.kind(),
            hidden: prop.hidden,
          });
          if (realProp.kind() == "object" && prop.repeated == false) {
            const rProp = realProp as PropObject;
            let newProps = rProp.properties.attrs.map(prop => {
              return { prop, path: _.clone(path) };
            });
            for (let nProp of newProps) {
              objectProperties.push(nProp);
            }
          }
        } else {
          if (prop.kind() == "object" && prop.repeated == false) {
            const rProp = prop as PropObject;
            let newProps = rProp.properties.attrs.map(prop => {
              return { prop, path: _.clone(path) };
            });
            for (let nProp of newProps) {
              objectProperties.push(nProp);
            }
          }
          result.push({
            id: `${this.id}-${path.join("-")}-${changeSetId}-${
              this.siStorable.updateClock.epoch
            }-${this.siStorable.updateClock.updateCount}`,
            name: prop.name,
            label: prop.label,
            path,
            prop,
            required: prop.required,
            repeated: prop.repeated,
            kind: prop.kind(),
            hidden: prop.hidden,
          });
        }
      }
      // This groups things according to their nesting, so we can just
      // walk the results and have everything in the proper order.
      const grouped = _.groupBy(result, value => {
        if (value.kind == "object") {
          return value.path;
        } else {
          return value.path.slice(0, -1);
        }
      });
      return _.flatten(Object.values(grouped));
    } else {
      let result: RegistryProperty[] = [];
      let path = entityProperty.path;
      path.push(`${index}`);
      result.push({
        id: `${this.id}-${path.join("-")}-${changeSetId}-${
          this.siStorable.updateClock.epoch
        }-${this.siStorable.updateClock.updateCount}`,
        name: entityProperty.name,
        label: entityProperty.label,
        path,
        prop: entityProperty.prop,
        required: entityProperty.required,
        repeated: entityProperty.repeated,
        kind: entityProperty.kind,
        hidden: entityProperty.hidden,
      });
      return result;
    }
  }

  async save(): Promise<void> {
    const currentObj = await db.nodes.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.nodes.put(this);
      await this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    await store.dispatch("editor/fromNode", this);
  }

  static async restore(): Promise<void> {
    let iObjects = await db.nodes.toArray();
    for (const iobj of iObjects) {
      let obj = new Node(iobj);
      await obj.dispatch();
    }
  }
}

db.nodes.mapToClass(Node);
