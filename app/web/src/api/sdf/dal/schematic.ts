import { NodeKind } from "@/api/sdf/dal/node";

export interface Schematic {
  poop: string;
}

export enum SchematicKind {
  Deployment = "deployment",
  Component = "component",
}

export function schematicKindFromNodeKind(kind: NodeKind): SchematicKind {
  switch (kind) {
    case NodeKind.Deployment:
      return SchematicKind.Deployment;
    case NodeKind.Component:
      return SchematicKind.Component;
  }
  throw Error(`Unknown NodeKind member: ${kind}`);
}

export function nodeKindFromSchematicKind(
  kind: SchematicKind | null,
): NodeKind | null {
  switch (kind) {
    case SchematicKind.Deployment:
      return NodeKind.Deployment;
    case SchematicKind.Component:
      return NodeKind.Component;
  }
  return null;
}

export function schematicKindFromString(s: string): SchematicKind {
  switch (s) {
    case "deployment":
      return SchematicKind.Deployment;
    case "component":
      return SchematicKind.Component;
  }
  throw Error(`Unknown SchematicKind member: ${s}`);
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
