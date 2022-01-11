export interface Schematic {
  poop: string;
}

export enum SchematicKind {
  Deployment = "deployment",
  Component = "component",
}

export function schematicKindFromString(s: string): SchematicKind {
  switch (s) {
    case "deployment":
      return SchematicKind.Deployment;
    case "component":
      return SchematicKind.Component;
    default:
      throw Error(`Unknown SchematicKind member: ${s}`);
  }
}

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

export interface EditorContext {
  applicationNodeId: number;
  systemId?: number;
}

export type MenuItem = Category | Item | LinkNodeItem;
