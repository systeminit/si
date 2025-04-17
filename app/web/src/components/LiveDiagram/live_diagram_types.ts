/**
 * LiveDiagram Types
 *
 * This file defines the core types used by the LiveDiagram components.
 * It includes both domain types (reflecting our application's model)
 * and layout types (used for ELK integration).
 */

import { Vector2d } from "konva/lib/types";
import {
  DiagramNodeData,
  DiagramGroupData,
  DiagramEdgeData,
  DiagramSocketData,
  DiagramViewData,
} from "@/components/ModelingDiagram/diagram_types";
import { ElkNode, ElkEdge, ElkPoint } from "./utils/ElkLayoutEngine";

/**
 * Domain Types - Represent the application objects in our LiveDiagram
 */

// Base interface for diagram elements
export interface LiveDiagramElement {
  id: string;
  type: LiveDiagramElementType;
  originalData: unknown; // Reference to original data from stores
}

// Types of diagram elements
export enum LiveDiagramElementType {
  NODE = "node",
  GROUP = "group",
  EDGE = "edge",
  SOCKET = "socket",
  VIEW = "view",
}

// Node in the diagram
export interface LiveDiagramNode extends LiveDiagramElement {
  type: LiveDiagramElementType.NODE;
  originalData: DiagramNodeData;
  position: Vector2d & { width: number; height: number };
  sockets: LiveDiagramSocket[];
  parentId?: string;
  title: string;
  subtitle?: string;
  color: string;
  icon?: string;
}

// Group in the diagram (frame)
export interface LiveDiagramGroup extends LiveDiagramElement {
  type: LiveDiagramElementType.GROUP;
  originalData: DiagramGroupData;
  position: Vector2d & { width: number; height: number };
  sockets: LiveDiagramSocket[];
  childIds: string[];
  title: string;
  color: string;
}

// Socket (port) on a node or group
export interface LiveDiagramSocket extends LiveDiagramElement {
  type: LiveDiagramElementType.SOCKET;
  originalData: DiagramSocketData;
  parentId: string;
  position: Vector2d;
  side: "left" | "right" | "top" | "bottom";
  label?: string;
  isRequired?: boolean;
  isManagement?: boolean;
}

// Edge connecting sockets
export interface LiveDiagramEdge extends LiveDiagramElement {
  type: LiveDiagramElementType.EDGE;
  originalData: DiagramEdgeData;
  fromSocketId: string;
  toSocketId: string;
  fromNodeId: string;
  toNodeId: string;
  points: Vector2d[];
  isManagement?: boolean;
  isBidirectional?: boolean;
}

// View node
export interface LiveDiagramView extends LiveDiagramElement {
  type: LiveDiagramElementType.VIEW;
  originalData: DiagramViewData;
  position: Vector2d & { width: number; height: number };
  title: string;
  color: string;
}

// Union type for any diagram element
export type LiveDiagramAnyElement =
  | LiveDiagramNode
  | LiveDiagramGroup
  | LiveDiagramSocket
  | LiveDiagramEdge
  | LiveDiagramView;

// Union type for any container element
export type LiveDiagramContainer =
  | LiveDiagramNode
  | LiveDiagramGroup
  | LiveDiagramView;

/**
 * Layout Types - Used for configuring and receiving ELK layout results
 */

// Layout configuration for a node/component
export interface LiveNodeLayoutConfig {
  id: string;
  width: number;
  height: number;
  x?: number;
  y?: number;
  parentId?: string;
  type: "node" | "group" | "view";
  // Additional layout options specific to this node
  options?: {
    fixed?: boolean; // Whether position is fixed
    padding?: number; // Internal padding
    [key: string]: unknown;
  };
}

// Layout configuration for a socket/port
export interface LiveSocketLayoutConfig {
  id: string;
  parentId: string;
  side: "left" | "right" | "top" | "bottom";
  index?: number; // Position index among sockets on the same side
}

// Layout configuration for an edge
export interface LiveEdgeLayoutConfig {
  id: string;
  fromSocketId: string;
  toSocketId: string;
  fromNodeId: string;
  toNodeId: string;
  type?: "standard" | "management";
  // Additional layout options for this edge
  options?: {
    routing?: "orthogonal" | "polyline" | "splines";
    [key: string]: unknown;
  };
}

// Global layout configuration
export interface LiveLayoutConfig {
  algorithm?: "layered" | "force" | "stress" | "mrtree" | "radial";
  direction?: "DOWN" | "RIGHT" | "LEFT" | "UP";
  nodeSeparation?: number;
  rankSeparation?: number;
  padding?: number;
  edgeRouting?: "orthogonal" | "polyline" | "splines";
  aspectRatio?: number;
  hierarchical?: boolean;
  [key: string]: string | number | boolean | undefined;
}

// Complete layout request containing all elements
export interface LiveLayoutRequest {
  nodes: LiveNodeLayoutConfig[];
  edges: LiveEdgeLayoutConfig[];
  sockets: LiveSocketLayoutConfig[];
  config: LiveLayoutConfig;
}

/**
 * Event Types - Define events for LiveDiagram interactions
 */

// Base event type
export interface LiveDiagramEvent {
  type:
    | "select"
    | "hover"
    | "dragStart"
    | "dragMove"
    | "dragEnd"
    | "connect"
    | "disconnect"
    | "resize";
  element: LiveDiagramAnyElement;
  originalEvent?: MouseEvent;
}

// Event for node selection
export interface LiveDiagramSelectEvent extends LiveDiagramEvent {
  type: "select";
  multiple?: boolean; // Whether to add to selection or replace
}

// Event for drag operations
export interface LiveDiagramDragEvent extends LiveDiagramEvent {
  type: "dragStart" | "dragMove" | "dragEnd";
  position: Vector2d;
  delta?: Vector2d;
}

// Event for socket connection
export interface LiveDiagramConnectEvent extends LiveDiagramEvent {
  type: "connect" | "disconnect";
  sourceSocket: LiveDiagramSocket;
  targetSocket?: LiveDiagramSocket;
}

// Event for node resizing
export interface LiveDiagramResizeEvent extends LiveDiagramEvent {
  type: "resize";
  newSize: { width: number; height: number };
  direction: "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";
}

// Factory functions to convert from store data to LiveDiagram types
export const LiveDiagramFactory = {
  // Create a LiveDiagramNode from DiagramNodeData
  createNode(
    nodeData: DiagramNodeData,
    position?: Vector2d & { width: number; height: number },
  ): LiveDiagramNode {
    return {
      id: nodeData.def.id,
      type: LiveDiagramElementType.NODE,
      originalData: nodeData,
      position: position || {
        x: 0,
        y: 0,
        width: 200,
        height: 100, // Default size
      },
      sockets: [],
      parentId: nodeData.def.parentId,
      title: nodeData.def.title,
      subtitle: nodeData.def.subtitle || undefined,
      color: nodeData.def.color,
      icon: nodeData.def.typeIcon || nodeData.def.icon,
    };
  },

  // Create a LiveDiagramGroup from DiagramGroupData
  createGroup(
    groupData: DiagramGroupData,
    position?: Vector2d & { width: number; height: number },
  ): LiveDiagramGroup {
    return {
      id: groupData.def.id,
      type: LiveDiagramElementType.GROUP,
      originalData: groupData,
      position: position || {
        x: 0,
        y: 0,
        width: 400,
        height: 300, // Default size
      },
      sockets: [],
      childIds: groupData.def.childIds || [],
      title: groupData.def.title,
      color: groupData.def.color,
    };
  },

  // Create a LiveDiagramSocket from DiagramSocketData
  createSocket(
    socketData: DiagramSocketData,
    parentId: string,
    position?: Vector2d,
  ): LiveDiagramSocket {
    return {
      id: socketData.def.id,
      type: LiveDiagramElementType.SOCKET,
      originalData: socketData,
      parentId,
      position: position || { x: 0, y: 0 },
      side: socketData.def.nodeSide,
      label: socketData.def.label,
      isRequired: socketData.def.isRequired,
      isManagement: socketData.def.isManagement,
    };
  },

  // Create a LiveDiagramEdge from DiagramEdgeData
  createEdge(
    edgeData: DiagramEdgeData,
    points: Vector2d[] = [],
  ): LiveDiagramEdge {
    return {
      id: edgeData.def.id,
      type: LiveDiagramElementType.EDGE,
      originalData: edgeData,
      fromSocketId: edgeData.def.fromSocketId,
      toSocketId: edgeData.def.toSocketId,
      fromNodeId: edgeData.def.fromComponentId,
      toNodeId: edgeData.def.toComponentId,
      points:
        points.length > 0
          ? points
          : [
              { x: 0, y: 0 },
              { x: 100, y: 100 },
            ],
      isManagement: edgeData.def.isManagement,
      isBidirectional: edgeData.def.isBidirectional,
    };
  },

  // Create a LiveDiagramView from DiagramViewData
  createView(
    viewData: DiagramViewData,
    position?: Vector2d & { width: number; height: number },
  ): LiveDiagramView {
    return {
      id: viewData.def.id,
      type: LiveDiagramElementType.VIEW,
      originalData: viewData,
      position: position || {
        x: 0,
        y: 0,
        width: 200,
        height: 100, // Default size
      },
      title: viewData.def.name,
      color: viewData.def.color,
    };
  },
};

// Mapper functions for ELK integration
export const ElkMapper = {
  // Convert LiveDiagramNode to ElkNode
  nodeToElk(node: LiveDiagramNode | LiveDiagramGroup): ElkNode {
    return {
      id: node.id,
      width: node.position.width,
      height: node.position.height,
      // Add ELK-specific properties as needed
    };
  },

  // Convert LiveDiagramEdge to ElkEdge
  edgeToElk(edge: LiveDiagramEdge): ElkEdge {
    return {
      id: edge.id,
      sources: [edge.fromSocketId],
      targets: [edge.toSocketId],
      // Add ELK-specific properties as needed
    };
  },

  // Convert ELK point array to LiveDiagramEdge points
  elkPointsToEdgePoints(points: ElkPoint[]): Vector2d[] {
    return points.map((p) => ({ x: p.x, y: p.y }));
  },
};
