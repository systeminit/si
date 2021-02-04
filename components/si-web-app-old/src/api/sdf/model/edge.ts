import _ from "lodash";

import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import { IListRequest, IListReply } from "@/api/sdf/model";
import {
  Query,
  Comparison,
  FieldType,
  BooleanTerm,
} from "@/api/sdf/model/query";
import { sdf } from "@/api/sdf";
import Bottle from "bottlejs";

export interface IAllPredecessorsRequest {
  objectId?: string;
  nodeId?: string;
  edgeKind: EdgeKind;
}

export interface IAllPredecessorsReply {
  edges: IEdge[];
}

export interface IAllSuccessorsRequest {
  objectId?: string;
  nodeId?: string;
  edgeKind: EdgeKind;
}

export interface IAllSuccessorsReply {
  edges: IEdge[];
}

export interface IVertex {
  nodeId: string;
  objectId: string;
  socket: string;
  typeName: string;
}

export enum EdgeKind {
  Configures = "configures",
  Includes = "includes",
}

export interface IEdge {
  id: string;
  tailVertex: IVertex;
  headVertex: IVertex;
  bidirectional: boolean;
  kind: EdgeKind;
  siStorable: ISiStorable;
}

export class Edge implements IEdge {
  id: IEdge["id"];
  tailVertex: IEdge["tailVertex"];
  headVertex: IEdge["headVertex"];
  bidirectional: IEdge["bidirectional"];
  kind: IEdge["kind"];
  siStorable: IEdge["siStorable"];

  constructor(args: IEdge) {
    this.id = args.id;
    this.tailVertex = args.tailVertex;
    this.headVertex = args.headVertex;
    this.bidirectional = args.bidirectional;
    this.kind = args.kind;
    this.siStorable = args.siStorable;
  }

  static upgrade(obj: Edge | IEdge): Edge {
    if (obj instanceof Edge) {
      return obj;
    } else {
      return new Edge(obj);
    }
  }

  static async allPredecessors(
    request: IAllPredecessorsRequest,
  ): Promise<Edge[]> {
    const items: Edge[] = [];

    if (request.nodeId) {
      const vertexes_to_check = [request.nodeId];

      for (let x = 0; x < vertexes_to_check.length; x++) {
        await db.edges
          .where({
            kind: request.edgeKind,
            "headVertex.nodeId": vertexes_to_check[x],
          })
          .each(edge => {
            vertexes_to_check.push(edge.tailVertex.nodeId);
            items.push(Edge.upgrade(edge));
          });
      }
    } else {
      const vertexes_to_check = [request.objectId];

      for (let x = 0; x < vertexes_to_check.length; x++) {
        await db.edges
          .where({
            kind: request.edgeKind,
            "headVertex.objectId": vertexes_to_check[x],
          })
          .each(edge => {
            vertexes_to_check.push(edge.tailVertex.objectId);
            items.push(Edge.upgrade(edge));
          });
      }
    }

    if (items.length) {
      return _.filter(items, e => !e.siStorable.deleted);
    }

    const reply: IAllPredecessorsReply = await sdf.get(
      "edges/allPredecessors",
      request,
    );
    for (let item of reply.edges) {
      const obj = new Edge(item);
      obj.save();
      items.push(obj);
    }
    return items;
  }

  static async directSuccessors(
    request: IAllSuccessorsRequest,
  ): Promise<Edge[]> {
    const items: Edge[] = [];

    if (request.nodeId) {
      const vertexes_to_check = [request.nodeId];

      for (let x = 0; x < vertexes_to_check.length; x++) {
        await db.edges
          .where({
            kind: request.edgeKind,
            "tailVertex.nodeId": vertexes_to_check[x],
          })
          .each(edge => {
            items.push(Edge.upgrade(edge));
          });
      }
    } else {
      const vertexes_to_check = [request.objectId];

      for (let x = 0; x < vertexes_to_check.length; x++) {
        await db.edges
          .where({
            kind: request.edgeKind,
            "tailVertex.objectId": vertexes_to_check[x],
          })
          .each(edge => {
            items.push(Edge.upgrade(edge));
          });
      }
    }

    if (items.length) {
      return _.filter(items, e => !e.siStorable.deleted);
    }

    const reply: IAllSuccessorsReply = await sdf.get(
      "edges/allSuccessors",
      request,
    );
    for (let item of reply.edges) {
      const obj = new Edge(item);
      obj.save();
      items.push(obj);
    }
    return items;
  }

  static async allSuccessors(request: IAllSuccessorsRequest): Promise<Edge[]> {
    const items: Edge[] = [];

    if (request.nodeId) {
      const vertexes_to_check = [request.nodeId];

      for (let x = 0; x < vertexes_to_check.length; x++) {
        await db.edges
          .where({
            kind: request.edgeKind,
            "tailVertex.nodeId": vertexes_to_check[x],
          })
          .each(edge => {
            if (!vertexes_to_check.includes(edge.headVertex.nodeId)) {
              vertexes_to_check.push(edge.headVertex.nodeId);
              // This doesn't address why there are duplicate edges in db.edges.
              // But it prevents duplication of nodes, at the moment.
              // TOTO - investigate why there are duplicate edges in db.edges.
              items.push(Edge.upgrade(edge));
            }
          });
      }
    } else {
      const vertexes_to_check = [request.objectId];

      for (let x = 0; x < vertexes_to_check.length; x++) {
        await db.edges
          .where({
            kind: request.edgeKind,
            "tailVertex.objectId": vertexes_to_check[x],
          })
          .each(edge => {
            if (!vertexes_to_check.includes(edge.headVertex.objectId)) {
              vertexes_to_check.push(edge.headVertex.objectId);
            }
            items.push(Edge.upgrade(edge));
          });
      }
    }

    if (items.length) {
      return _.filter(items, e => !e.siStorable.deleted);
    }

    const reply: IAllSuccessorsReply = await sdf.get(
      "edges/allSuccessors",
      request,
    );
    for (let item of reply.edges) {
      const obj = new Edge(item);
      obj.save();
      items.push(obj);
    }
    return items;
  }

  static async byTailTypeAndHeadType(
    kind: IEdge["kind"],
    tailTypeName: string,
    headTypeName: string,
  ): Promise<Edge[]> {
    let items = await db.edges
      .where({
        kind,
        "tailVertex.typeName": tailTypeName,
        "headVertex.typeName": headTypeName,
      })
      .toArray();
    if (!items.length) {
      const results = await Edge.list({
        query: new Query({
          booleanTerm: BooleanTerm.And,
          items: [
            {
              expression: {
                field: "kind",
                value: kind.toString(),
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
            {
              expression: {
                field: "tailVertex.typeName",
                value: tailTypeName,
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
            {
              expression: {
                field: "headVertex.typeName",
                value: headTypeName,
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
          ],
        }),
      });
      return results.items;
    } else {
      return items.map(obj => new Edge(obj));
    }
  }

  static async byTailTypeForHeadObjectId(
    kind: IEdge["kind"],
    tailTypeName: string,
    headObjectId: string,
  ): Promise<Edge[]> {
    let items = await db.edges
      .where({
        kind,
        "tailVertex.typeName": tailTypeName,
        "headVertex.objectId": headObjectId,
      })
      .toArray();
    if (!items.length) {
      const results = await Edge.list({
        query: new Query({
          booleanTerm: BooleanTerm.And,
          items: [
            {
              expression: {
                field: "kind",
                value: kind.toString(),
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
            {
              expression: {
                field: "tailVertex.typeName",
                value: tailTypeName,
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
            {
              expression: {
                field: "headVertex.objectId",
                value: headObjectId,
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
          ],
        }),
      });
      return results.items;
    } else {
      return items.map(obj => new Edge(obj));
    }
  }

  static async byVertexTypes(
    kind: IEdge["kind"],
    tailTypeName: string,
    headTypeName: string,
  ): Promise<Edge[]> {
    let items = await db.edges
      .where({
        kind,
        "tailVertex.typeName": tailTypeName,
        "headVertex.typeName": headTypeName,
      })
      .toArray();
    if (!items.length) {
      const results = await Edge.list({
        query: new Query({
          booleanTerm: BooleanTerm.And,
          items: [
            {
              expression: {
                field: "kind",
                value: kind.toString(),
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
            {
              expression: {
                field: "tailVertex.typeName",
                value: tailTypeName,
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
            {
              expression: {
                field: "headVertex.typeName",
                value: headTypeName,
                comparison: Comparison.Equals,
                fieldType: FieldType.String,
              },
            },
          ],
        }),
      });
      return results.items;
    } else {
      return items.map(obj => new Edge(obj));
    }
  }

  static async list(request?: IListRequest): Promise<IListReply<Edge>> {
    const items: Edge[] = [];
    let totalCount = 0;

    if (!request) {
      await db.edges.each(obj => {
        items.push(new Edge(obj));
        totalCount++;
      });
    }

    if (!totalCount) {
      let finished = false;
      while (!finished) {
        const reply: IListReply<IEdge> = await sdf.list("edges", request);
        if (reply.items.length) {
          for (let item of reply.items) {
            let objItem = new Edge(item);
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

  async delete(): Promise<void> {
    this.siStorable.deleted = true;
    this.save();
    await sdf.delete(`edges/${this.id}`);
  }

  async save(): Promise<void> {
    const currentObj = await db.edges.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.edges.put(this);
      this.dispatch();
    }
  }

  async dispatch(): Promise<void> {
    const bottle = Bottle.pop("default");
    const store = bottle.container.Store;
    await store.dispatch("application/fromEdge", this, { root: true });
    await store.dispatch("editor/fromEdge", this, { root: true });
  }

  static async restore(): Promise<void> {
    let iObjects = await db.edges.toArray();
    for (const iobj of iObjects) {
      let obj = new Edge(iobj);
      await obj.dispatch();
    }
  }
}

db.edges.mapToClass(Edge);
