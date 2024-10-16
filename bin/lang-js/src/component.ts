export interface Component {
  name: string;
  properties: Record<string, unknown>;
}

export interface ComponentWithGeometry {
  properties: Record<string, unknown>;
  geometry: {
    x: string,
    y: string,
    width?: string,
    height?: string,
  }
}
