export interface NodeCreate {
  rootNodeId: number;
  nodeSchemaId: number;
  systemId?: number;
  x: string;
  y: string;
  parentNodeId?: number;
}
