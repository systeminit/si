import { db } from "@/api/sdf/dexie";
import { ISiStorable } from "@/api/sdf/model/siStorable";
import store from "@/store";
import _ from "lodash";

export interface IVertex {
  nodeId: string;
  objectId: string;
  socket: string;
  typeName: string;
}

export type EdgeKind = "includes" | IEdgeKindConfigures;
export interface IEdgeKindConfigures {
  configures: {
    systemIds: string[];
  };
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
