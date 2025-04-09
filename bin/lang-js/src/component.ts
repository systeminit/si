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
  geometry: { [key: string]: Geometry };
}
