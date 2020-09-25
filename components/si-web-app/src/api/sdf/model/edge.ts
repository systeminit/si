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
import store from "@/store";
import { sdf } from "@/api/sdf";

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

  async save(): Promise<void> {
    const currentObj = await db.edges.get(this.id);
    if (!_.eq(currentObj, this)) {
      await db.edges.put(this);
      if (
        this.tailVertex.typeName == "system" &&
        this.headVertex.typeName == "application"
      ) {
        await store.dispatch("application/fromEdge", this, { root: true });
      }
    }
  }
}

db.edges.mapToClass(Edge);
