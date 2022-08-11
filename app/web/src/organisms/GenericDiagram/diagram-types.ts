export type DiagramConfig = {
  // canNodesConnectToThemselves: boolean;
};

export type GridPoint = { x: number; y: number };

export type DiagramNodeDef = {
  /** identifier for the node */
  id: string;
  /** node type within the context of the diagram */
  type?: string;
  /** name of diagram node */
  name: string;
  /** subtitle of diagram node */
  subtitle?: string;
  /** more text content displayed within the node */
  content?: string;
  /** sockets on the node */
  sockets?: DiagramSocketDef[];
  /** x,y placement of the node on the diagram */
  position: GridPoint;
  // statusIcons?: NodeStatusIcon[];
  // color
};

export type DiagramSocketDef = {
  id: string;
  type?: string;
  name: string;
  canConnectToSocketType: string[];
  arity: "one" | "many";
  maxConnections?: number;
  isRequired?: boolean;
  nodeSide: "left" | "right"; // add top/bottom later?
  // color
  // shape
};

export type DiagramEdgeDef = {
  id: string;
  type?: string;
  name?: string;
  fromSocketId: string;
  toSocketId: string;
  isBidirectional?: boolean;
  // color
  // thickness
};

export type DiagramNodeGroupDef = {
  id: string;
  type?: string;
  name: string;
  isCollapsible?: boolean;
  nodeIds: string[];
  sockets?: DiagramSocketDef[];
};
