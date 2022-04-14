import { Color } from "./common";
import { SchematicKind } from "@/api/sdf/dal/schematic";

/**  Display properties */
interface SocketDisplay {
  color?: Color;
}

/** a Socket */
export interface Socket {
  id: string;
  name?: string;
  type: string;
  display?: SocketDisplay;
  schematic_kind: SchematicKind;
}

export interface SocketCtx {
  nodeId: string;
  socketId: string;
}
