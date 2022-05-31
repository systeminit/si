export interface NodeCreate {
  nodeSchemaId: number;
  systemId?: number;
  x: string;
  y: string;
  parentNodeId?: number;
}

export interface ConnectionCreate {
  sourceNodeId: number;
  sourceSocketId: number;
  destinationNodeId: number;
  destinationSocketId: number;
}
