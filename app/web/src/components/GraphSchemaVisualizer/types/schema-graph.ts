import type { EntityKind } from "@/workers/types/entity_kind_types";

// Generic entity interface for input data
export interface Entity {
  id: string;
  name: string;
  entityKind: EntitySchemaKind;
  properties: EntityProperty[];
}

export enum EntitySchemaKind {
  Action = 'Action',
  Component = 'Component',
  SchemaVariant = 'SchemaVariant',
  Schema = 'Schema',
  Function = 'Function',
}

// Core entity interfaces for internal use
export type EntityNode = Entity & {
  position: { x: number; y: number };
  dimensions: { width: number; height: number };
}

export type EntityProperty = {
  kind: "primary_key" | "value",
  type: string,
  name: string,
} | {
  kind: "foreign_key",
  type: string,
  arity: 'one' | 'many',
  name: string, // matches a primary key in another entity
}

export interface RelationshipEdge {
  id: string;
  sourceId: string;
  targetId: string;
  arity: 'one' | 'many';
  sourceProperty?: string;
  targetProperty?: string;
}

export interface GraphData {
  nodes: EntityNode[];
  edges: RelationshipEdge[];
}

export interface GraphViewportState {
  x: number;
  y: number;
  scale: number;
}

export interface GraphInteractionState {
  selectedEntities: Set<string>;
  hoveredEntity: string | null;
  searchQuery: string;
  viewportState: GraphViewportState;
}

// Simplified entity data structure
export interface EntityData {
  entities: Entity[];
  relationships: RelationshipEdge[];
}

// Layout configuration
export interface LayoutOptions {
  algorithm: 'hierarchical' | 'layered' | 'force';
  spacing: {
    nodeNode: number;
    layerSeparation: number;
    portPort: number;
  };
  direction: 'UP' | 'DOWN' | 'LEFT' | 'RIGHT';
}

// Graph props for the main component
export interface GraphSchemaVisualizerProps {
  selectedEntity?: string;
  showRelationships?: boolean;
  entityTypes?: EntitySchemaKind[];
}