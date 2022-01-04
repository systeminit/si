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
  entityType: string;
  links?: LinkNodeItem[];
}

export interface LinkNodeItem {
  kind: "link";
  entityType: string;
  nodeId: string;
  entityId: string;
  name: string;
}

export type MenuItem = Category | Item | LinkNodeItem;
