import { Color, SchematicObject } from "./common";

/**  Replace string for correct connecion tyoe */
export enum ConnectionKind {
  CONFIGURES = "configures",
  INCLUDES = "includes",
  DEPLOYMENT = "deployment",
  IMPLEMENTATION = "implementation",
}

interface ConnectionClassification {
  kind: ConnectionKind;
}

/**  Display properties */
interface ConnectionDisplay {
  color: Color;
}

interface Source {
  nodeId: string;
  socketId: string;
}

interface Destination {
  nodeId: string;
  socketId: string;
}

/**  A connection (from an output to an input) */
export interface Connection extends SchematicObject {
  id: string;
  classification: ConnectionClassification;
  source: Source;
  destination: Destination;
  display?: ConnectionDisplay;
}

export interface ConnectionCreate {
  id: string;
}
