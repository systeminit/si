import _ from "lodash";

import { ISiStorable } from "@/api/sdf/model/siStorable";

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
}
