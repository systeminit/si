import { getSchematic } from "./schematic/get_schematic";
import { setSchematic } from "./schematic/set_schematic";
import { setNode } from "./schematic/set_node";
import { createConnection } from "./schematic/create_connection";

export const SchematicService = {
  getSchematic,
  setSchematic,
  setNode,
  createConnection,
};
