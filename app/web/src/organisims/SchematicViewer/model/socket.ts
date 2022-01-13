import { Color } from "./common";

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
}

export interface SocketCtx {
  nodeId: string;
  socketId: string;
}
