import { getSchematic } from "./schematic/get_schematic";
import { setSchematic } from "./schematic/set_schematic";
import { setNode } from "./schematic/set_node";
import { createConnection } from "./schematic/create_connection";
import { getNodeAddMenu } from "./schematic/get_node_add_menu";

export const SchematicService = {
  getSchematic,
  setSchematic,
  getNodeAddMenu,
  setNode,
  createConnection,
};
