export interface NodeCreate {
  nodeSchemaId: number;
  systemId?: number;
  x: string;
  y: string;
  parentNodeId: number | null;
}
