import { getSchematic } from "./schematic/get_schematic";
import { createConnection } from "./schematic/create_connection";
import { getNodeAddMenu } from "./schematic/get_node_add_menu";
import { getNodeTemplate } from "./schematic/get_node_template";
import { createNode } from "./schematic/create_node";
import { setNodePosition } from "./schematic/set_node_position";

export const SchematicService = {
  getSchematic,
  getNodeAddMenu,
  getNodeTemplate,
  createConnection,
  createNode,
  setNodePosition,
};
