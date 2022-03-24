import { Color, SchematicObject } from "./common";

/**  Replace string for correct connecion type */
export enum ConnectionKind {
  CONFIGURES = "configures",
  INCLUDES = "includes",
  DEPLOYMENT = "deployment",
  IMPLEMENTATION = "implementation",
}

// Backend schema doesn't match frontend schema.
// Updating to match what the backend is sending.
// interface ConnectionClassification {
//   kind: ConnectionKind;
// }
type ConnectionClassification = ConnectionKind;

/**  Display properties */
interface ConnectionDisplay {
  color: Color;
}

interface Source {
  nodeId: number;
  socketId: string;
}

interface Destination {
  nodeId: number;
  socketId: string;
}

/**  A connection (from an output to an input) */
export interface Connection extends SchematicObject {
  id: number;
  classification: ConnectionClassification;
  source: Source;
  destination: Destination;
  // FIXME(nick): the backend is not returning or storing color at this time.
  display?: ConnectionDisplay;
}

export interface ConnectionCreate {
  sourceNodeId: number;
  sourceSocketId: number;
  destinationNodeId: number;
  destinationSocketId: number;
}
