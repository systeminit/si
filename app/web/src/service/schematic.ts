import { getSchematic } from "./schematic/get_schematic";
import { getSchematic2 } from "./schematic/get_schematic2";
import { createConnection } from "./schematic/create_connection";
import { getNodeAddMenu } from "./schematic/get_node_add_menu";
import { getNodeTemplate } from "./schematic/get_node_template";
import { createNode } from "./schematic/create_node";
import { setNodePosition } from "./schematic/set_node_position";
import { listSchemaVariants } from "./schematic/list_schema_variants";

export const SchematicService = {
  getSchematic,
  getSchematic2,
  getNodeAddMenu,
  getNodeTemplate,
  createConnection,
  createNode,
  setNodePosition,
  listSchemaVariants,
};
