import { SchematicData } from "./common";
import { Node } from "./node";
import { Connection } from "./connection";

export interface Schematic extends SchematicData {
  nodes: Node[];
  connections: Connection[];
}
