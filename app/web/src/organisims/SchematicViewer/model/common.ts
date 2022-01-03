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

interface SchematicParticipation {
  deployment: boolean;
  component: boolean;
}

export interface SchematicObject extends SchematicData {
  schematic: SchematicParticipation;
}
