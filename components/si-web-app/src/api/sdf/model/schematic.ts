import { Node, INodeObject } from "@/api/sdf/model/node";
import { Edge } from "@/api/sdf/model/edge";

export enum SchematicKind {
  Deployment = "deployment",
  Component = "component",
}

export interface IConnectionEdge {
  edgeId: string;
  nodeId: string;
  socketId: string;
}

export interface IConnections {
  predeccessors: {
    [edgeKind: string]: IConnectionEdge[];
  };
  successors: {
    [edgeKind: string]: IConnectionEdge[];
  };
}

export enum ISocketKind {
  Input = "input",
  Output = "output",
}

export enum ISocketTypes {
  Object = "object",
}

export interface ISocketObject {
  id: string;
  kind: ISocketKind;
  type: "object";
  objectType: string;
}

export interface ISocketPrimitive {
  id: string;
  socketKind: ISocketKind;
  socketType: string;
  objectType?: never;
}

export type ISocket = ISocketObject | ISocketPrimitive;

export interface ISchematicNode {
  node: Node;
  sockets: {
    inputs: ISocket[];
    outputs: ISocket[];
  };
  object: INodeObject;
  connections: IConnections;
}

export interface ISchematic {
  nodes: {
    [nodeId: string]: ISchematicNode;
  };
  edges: {
    [edgeId: string]: Edge;
  };
}

export class Schematic implements ISchematic {
  kind?: SchematicKind;
  nodes: ISchematic["nodes"];
  edges: ISchematic["edges"];

  constructor(schematic: ISchematic) {
    this.nodes = schematic.nodes;
    this.edges = schematic.edges;
  }
}
