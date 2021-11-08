import { Edge, EdgeKind } from "@/api/sdf/model/edge";

export interface ConnectionPoint {
  nodeId: String;
  nodeName: String;
  nodeDescription: String;
  nodeType: String;
  socketName: String;
  socketType: String;
}

export interface Connection {
  edge: Edge;
  kind: EdgeKind;
  source: ConnectionPoint;
  destination: ConnectionPoint;
}
