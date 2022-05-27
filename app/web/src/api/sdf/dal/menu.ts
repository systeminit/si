import { SchematicKind } from "@/api/sdf/dal/schematic";

export interface MenuFilter {
  schematicKind: SchematicKind;
  rootComponentId: number;
}

export interface Category {
  kind: "category";
  name: string;
  items: MenuItem[];
}

export interface Item {
  kind: "item";
  name: string;
  schema_id: number;
  links?: LinkNodeItem[];
}

// TODO: This entire thing is wrong now, but should look like item eventually. -- Adam
export interface LinkNodeItem {
  kind: "link";
  entityType: string;
  nodeId: string;
  entityId: string;
  name: string;
}

export type MenuItem = Category | Item | LinkNodeItem;
