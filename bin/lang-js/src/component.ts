import { SocketName, SocketRefAndValue } from "./function_kinds/management.ts";

export interface Component {
  name: string;
  properties: Record<string, unknown>;
}

export interface Geometry {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
}

export interface ComponentWithGeometry {
  properties: Record<string, unknown>;
  sources: Record<string, { component: string; path: string; func?: string }>;
  geometry: { [key: string]: Geometry };
  incomingConnections?: {
    [key: SocketName]: SocketRefAndValue[] | SocketRefAndValue | undefined;
  };
}
