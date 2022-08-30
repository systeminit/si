import { getNodeAddMenu } from "./diagram/get_node_add_menu";
import { getNodeTemplate } from "./diagram/get_node_template";
import { createNode } from "./diagram/create_node";
import { setNodePosition } from "./diagram/set_node_position";
import { listSchemaVariants } from "./diagram/list_schema_variants";

export const DiagramService = {
  getNodeAddMenu,
  getNodeTemplate,
  createNode,
  setNodePosition,
  listSchemaVariants,
};
