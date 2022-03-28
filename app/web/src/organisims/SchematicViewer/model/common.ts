/** hexadecimal */
export type Color = number;

export interface Position {
  x: number;
  y: number;
}

export interface SchematicData {
  lastUpdated: Date;
  checksum: string;
}
