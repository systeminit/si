import { Vector2d } from "konva/lib/types";
import { IconNames } from "@/ui-lib/icons/icon_set";

export type DiagramConfig = {
  // canNodesConnectToThemselves: boolean;
  icons?: Record<string, string>;
  toneColors?: Record<string, string>;
};

export type GridPoint = { x: number; y: number };
export type Direction = "up" | "down" | "left" | "right";

export type DiagramElementTypes = "node" | "socket" | "edge";

export type DiagramElementIdentifier = {
  diagramElementType: DiagramElementTypes;
  id: string;
};

export type DiagramStatusIcon = {
  /* name/id of icon (registered in diagram config) */
  icon: IconNames;
  /* tone of icon - gets mapped to some preset colors */
  tone?: "success" | "error" | "warning" | "info" | "neutral";
  /* set to override specific hex color */
  color?: string;
};

export type DiagramNodeDef = {
  /** unique id of the node */
  id: string;
  /** node type within the context of the diagram */
  type?: string | null;
  /** category of diagram node */
  category?: string | null;
  /** title of diagram node */
  title: string;
  /** subtitle of diagram node */
  subtitle?: string | null;
  /** more text content displayed within the node */
  content?: string | null;
  /** sockets on the node */
  sockets?: DiagramSocketDef[];
  /** x,y placement of the node on the diagram */
  position: GridPoint;
  /** single hex color to use for node theme */
  color?: string | null;
  /** icon (name/slug) used to help convey node type */
  typeIcon?: string | null;
  /** array of icons (slug and colors) to show statuses */
  statusIcons?: DiagramStatusIcon[];
  /** if true, node shows the `loading` overlay */
  isLoading: boolean;
};

export type DiagramSocketDef = {
  /** unique id of the socket - should be unique across all sockets */
  id: string;
  /** label displayed with the socket */
  label: string;
  /** type - will only connect to sockets of the same type */
  type: string;
  /** direction of data flow from this socket */
  direction: "input" | "output" | "bidirectional";
  /** arity / max number of connections - null = no limit (most will likely be either 1 or null) */
  maxConnections: number | null;
  /** is a connection required to be valid - will affect display */
  isRequired?: boolean;
  /** which side of the node is the socket displayed on */
  nodeSide: "left" | "right"; // add top/bottom later?

  // color
  // shape
};

export type DiagramContent = {
  nodes: DiagramNodeDef[];
  edges: DiagramEdgeDef[];
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

// specific features... likely will move these as the diagram functionality gets broken up
export type PendingInsertedElement = {
  diagramElementType: DiagramElementTypes;
  // TODO: will likely need more info here if you can insert more specific subtypes of things, so we can vary display a bit
  insertedAt?: Date;
  position?: Vector2d;
  temporaryId?: string;
};

export type DiagramDrawEdgeState = {
  active: boolean;
  fromSocketId?: string;
  toSocketId?: string;
  targetSocketIds: string[];
};

// Event payloads - emitted by generic diagram //////////////////////////////////
export type MoveElementEvent = {
  id: string;
  diagramElementType: DiagramElementTypes;
  position: Vector2d;
  isFinal: boolean;
};
export type DrawEdgeEvent = {
  fromSocketId: string;
  toSocketId: string;
};
export type DeleteElementsEvent = {
  elements: DiagramElementIdentifier[];
};
export type InsertElementEvent = {
  diagramElementType: DiagramElementTypes;
  position: Vector2d;
  onComplete: () => void;
};

export type RightClickElementEvent = {
  id: string;
  diagramElementType: DiagramElementTypes;
  e: MouseEvent;
};
