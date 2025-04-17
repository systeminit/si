// ElkLayoutEngine.ts - Interface to the ELK layout library
import ELK from "elkjs/lib/elk.bundled.js";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  DiagramNodeData,
  DiagramGroupData,
  DiagramEdgeData,
  DiagramSocketData,
} from "@/components/ModelingDiagram/diagram_types";

/**
 * ELK Graph Layout Interfaces
 * Based on: https://www.eclipse.org/elk/reference.html
 */

// ELK Core Interfaces for nodes, ports, edges
export interface ElkNode {
  id: string;
  width: number;
  height: number;
  x?: number;
  y?: number;
  children?: ElkNode[];
  ports?: ElkPort[];
}

export interface ElkPort {
  id: string;
  width: number;
  height: number;
  x?: number;
  y?: number;
  properties?: {
    side: "WEST" | "EAST" | "NORTH" | "SOUTH";
  };
}

export interface ElkEdge {
  id: string;
  sources: string[];
  targets: string[];
  sections?: ElkSection[];
}

export interface ElkSection {
  startPoint: ElkPoint;
  endPoint: ElkPoint;
  bendPoints?: ElkPoint[];
}

export interface ElkPoint {
  x: number;
  y: number;
}

export interface ElkGraph {
  id: string;
  children: ElkNode[];
  edges: ElkEdge[];
  layoutOptions?: Record<string, string | number | boolean | undefined>;
}

// Layout Options Interface
export interface ElkLayoutOptions {
  algorithm?: "layered" | "force" | "stress" | "mrtree" | "radial";
  direction?: "DOWN" | "RIGHT" | "LEFT" | "UP";
  aspectRatio?: number;
  spacing?: number;
  padding?: number;
  edgeRouting?: "ORTHOGONAL" | "POLYLINE" | "SPLINES";
  [key: string]: string | number | boolean | undefined;
}

/**
 * Layout Result Interfaces - what our components will use
 */
export interface LayoutResult {
  nodes: LayoutNode[];
  edges: LayoutEdge[];
}

export interface LayoutNode {
  id: string;
  position: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  sockets: LayoutSocket[];
  parentId?: string;
}

export interface LayoutSocket {
  id: string;
  position: {
    x: number;
    y: number;
  };
  nodeSide: "left" | "right" | "top" | "bottom";
}

export interface LayoutEdge {
  id: string;
  points: ElkPoint[];
  fromSocketId: string;
  toSocketId: string;
}

/**
 * ElkLayoutEngine - Main interface to the ELK layout library
 */
export class ElkLayoutEngine {
  private static elk: any;

  // Initialize ELK instance
  private static getElk() {
    if (!this.elk) {
      this.elk = new ELK();
    }
    return this.elk;
  }

  /**
   * Main layout function - computes positions for nodes and edges
   * @param components - Array of diagram components
   * @param edges - Array of diagram edges
   * @param options - Optional layout options
   */
  static async computeLayout(
    components: (DiagramNodeData | DiagramGroupData)[],
    edges: DiagramEdgeData[],
    options?: ElkLayoutOptions,
  ): Promise<LayoutResult> {
    // Create ELK graph structure
    const graph: ElkGraph = {
      id: "root",
      layoutOptions: {
        "elk.algorithm": "layered",
        "elk.direction": "DOWN",
        "elk.layered.spacing.nodeNodeBetweenLayers": "25",
        "elk.layered.spacing.compaction.connectedComponents": "true",
        "elk.spacing.nodeNode": "80",
        "elk.padding": "[50, 50, 50, 50]",
        "elk.edgeRouting": "ORTHOGONAL",
        "elk.layered.nodePlacement.strategy": "INTERACTIVE",
        "elk.layered.considerModelOrder.strategy": "NODES_AND_EDGES",
        "elk.layered.considerModelOrder": "true",
        "elk.hierarchyHandling": "INCLUDE_CHILDREN",
        "elk.layered.crossingMinimization.semiInteractive": "true",
        // "elk.separateConnectedComponents": "false",

        ...options,
      },
      children: [],
      edges: [],
    };

    // Add nodes to graph
    components.forEach((component) => {
      // Create ELK node
      const elkNode: ElkNode = {
        id: component.def.id,
        // Ensure width and height are numbers
        width:
          typeof component.def.width === "number"
            ? component.def.width
            : component.def.width
            ? parseFloat(String(component.def.width))
            : 200,
        height:
          typeof component.def.height === "number"
            ? component.def.height
            : component.def.height
            ? parseFloat(String(component.def.height))
            : 100,
        // Add initial positions if they exist in the input
        ...(component.def.x !== undefined && {
          x: parseFloat(String(component.def.x)),
        }),
        ...(component.def.y !== undefined && {
          y: parseFloat(String(component.def.y)),
        }),
        ports: [],
      };

      // If the node doesn't already have a position, ensure we make it different
      // from other nodes so they don't stack on top of each other
      if (elkNode.x === undefined || elkNode.y === undefined) {
        // Give each node a unique initial position based on its index in the array
        const index = graph.children.length;
        elkNode.x = (index % 5) * 250; // 5 nodes per row, 250px apart
        elkNode.y = Math.floor(index / 5) * 150; // 150px between rows
      }

      // Group sockets by side to distribute them evenly
      const socketsBySide = {
        left: [] as DiagramSocketData[],
        right: [] as DiagramSocketData[],
        top: [] as DiagramSocketData[],
        bottom: [] as DiagramSocketData[],
      };

      // First, group all sockets by their side
      component.sockets.forEach((socket) => {
        const side = socket.def.nodeSide;
        if (side in socketsBySide) {
          socketsBySide[side].push(socket);
        } else {
          // Default to right side if invalid side
          socketsBySide.right.push(socket);
        }
      });

      // Now add ports (sockets) with distributed positions
      Object.entries(socketsBySide).forEach(([side, sockets]) => {
        if (sockets.length === 0) return;

        const elkSide =
          side === "left"
            ? "WEST"
            : side === "right"
            ? "EAST"
            : side === "top"
            ? "NORTH"
            : "SOUTH";

        // Calculate distribution along the side
        sockets.forEach((socket, index) => {
          let portX = 0;
          let portY = 0;

          // For vertical sides (left/right), distribute ports evenly along the height
          if (side === "left" || side === "right") {
            const segment = (elkNode.height - 20) / (sockets.length + 1);
            portY = 10 + segment * (index + 1);
            portX = side === "left" ? 0 : elkNode.width;
          }
          // For horizontal sides (top/bottom), distribute ports evenly along the width
          else {
            const segment = (elkNode.width - 20) / (sockets.length + 1);
            portX = 10 + segment * (index + 1);
            portY = side === "top" ? 0 : elkNode.height;
          }

          const port: ElkPort = {
            id: socket.def.id,
            width: 10,
            height: 10,
            x: portX - 5, // Center the port around the position
            y: portY - 5,
            properties: {
              side: elkSide,
            },
          };
          elkNode.ports!.push(port);
        });
      });

      graph.children.push(elkNode);
    });

    // Add edges to graph
    edges.forEach((edge) => {
      // Find the component IDs for the source and target sockets
      const fromComponentId = edge.def.fromComponentId;
      const toComponentId = edge.def.toComponentId;

      console.log(`Edge ${edge.def.id}: ${fromComponentId} -> ${toComponentId} (sockets: ${edge.def.fromSocketId} -> ${edge.def.toSocketId})`);

      const elkEdge: ElkEdge = {
        id: edge.def.id,
        sources: [edge.def.fromComponentId],
        targets: [edge.def.toComponentId],
        // Adding extra information as debug properties
        properties: {
          "fromComponentId": fromComponentId,
          "toComponentId": toComponentId
        }
      };
      graph.edges.push(elkEdge);
    });

    try {
      // Get ELK instance
      const elk = this.getElk();

      // Compute layout
      const layoutGraph = await elk.layout(graph);

      // Convert result to our format
      return this.convertElkResultToLayoutResult(layoutGraph);
    } catch (error) {
      console.error("ELK layout error:", error);
      console.log("ELK layout error, using fallback layout:", error);
      // Fallback to placeholder layout if ELK fails
      return this.generatePlaceholderLayout(components, edges);
    }
  }

  /**
   * Computes layout for a specific parent node's children
   * @param parentId - ID of the parent component
   * @param components - Array of all diagram components
   * @param edges - Array of all diagram edges
   */
  static async computeSubgraphLayout(
    parentId: ComponentId,
    components: (DiagramNodeData | DiagramGroupData)[],
    edges: DiagramEdgeData[],
    options?: ElkLayoutOptions,
  ): Promise<LayoutResult> {
    // Filter components and edges to only include those in the subgraph
    const childComponents = components.filter(
      (c) =>
        c.def.parentId === parentId || (c.def.isGroup && c.def.id === parentId),
    );

    // Filter edges to only include those between children
    const childIds = new Set(childComponents.map((c) => c.def.id));
    const childEdges = edges.filter(
      (e) =>
        childIds.has(e.def.fromComponentId) &&
        childIds.has(e.def.toComponentId),
    );

    // Use the main layout function with subgraph-specific options
    const subgraphOptions: ElkLayoutOptions = {
      ...options,
      "elk.padding": "[20, 20, 20, 20]", // Add padding inside the parent
    };

    return this.computeLayout(childComponents, childEdges, subgraphOptions);
  }

  /**
   * Convert the ELK layout result to our LayoutResult format
   */
  private static convertElkResultToLayoutResult(
    elkGraph: ElkGraph,
  ): LayoutResult {
    const result: LayoutResult = {
      nodes: [],
      edges: [],
    };

    // Process nodes
    if (elkGraph.children) {
      elkGraph.children.forEach((elkNode) => {
        // Create layout node
        const layoutNode: LayoutNode = {
          id: elkNode.id,
          position: {
            // Ensure we have numbers, not strings or undefined
            x:
              typeof elkNode.x === "number"
                ? elkNode.x
                : parseFloat(String(elkNode.x)) || 0,
            y:
              typeof elkNode.y === "number"
                ? elkNode.y
                : parseFloat(String(elkNode.y)) || 0,
            width:
              typeof elkNode.width === "number"
                ? elkNode.width
                : parseFloat(String(elkNode.width)) || 200,
            height:
              typeof elkNode.height === "number"
                ? elkNode.height
                : parseFloat(String(elkNode.height)) || 100,
          },
          sockets: [],
        };

        // Log the actual position values to debug
        console.log(`ELK output for node ${elkNode.id}:`, {
          raw: {
            x: elkNode.x,
            y: elkNode.y,
            width: elkNode.width,
            height: elkNode.height,
          },
          processed: layoutNode.position,
        });

        // Group ports by side for better positioning
        const portsBySide: Record<string, ElkPort[]> = {
          WEST: [],
          EAST: [],
          NORTH: [],
          SOUTH: [],
        };

        // First, group all ports by their side
        if (elkNode.ports) {
          elkNode.ports.forEach((port) => {
            const side = port.properties?.side || "EAST";
            if (side in portsBySide) {
              portsBySide[side].push(port);
            } else {
              // Default to right side if invalid side
              portsBySide.EAST.push(port);
            }
          });
        }

        // Process ports (sockets) for each side
        Object.entries(portsBySide).forEach(([portSide, ports]) => {
          if (ports.length === 0) return;

          const nodeSide =
            portSide === "WEST"
              ? "left"
              : portSide === "EAST"
              ? "right"
              : portSide === "NORTH"
              ? "top"
              : "bottom";

          // Process each port for this side
          ports.forEach((port, index) => {
            // If port already has defined position from ELK, use it
            if (port.x !== undefined && port.y !== undefined) {
              layoutNode.sockets.push({
                id: port.id,
                position: {
                  x: port.x + port.width / 2,
                  y: port.y + port.height / 2,
                },
                nodeSide,
              });
            } else {
              // Otherwise, calculate a distributed position along the side
              let posX = 0;
              let posY = 0;

              if (portSide === "WEST" || portSide === "EAST") {
                // Distribute vertically for left/right sides
                const segment = elkNode.height / (ports.length + 1);
                posY = segment * (index + 1);
                posX = portSide === "WEST" ? 0 : elkNode.width;
              } else {
                // Distribute horizontally for top/bottom sides
                const segment = elkNode.width / (ports.length + 1);
                posX = segment * (index + 1);
                posY = portSide === "NORTH" ? 0 : elkNode.height;
              }

              layoutNode.sockets.push({
                id: port.id,
                position: { x: posX, y: posY },
                nodeSide,
              });
            }
          });
        });

        result.nodes.push(layoutNode);
      });
    }

    // Process edges
    if (elkGraph.edges) {
      elkGraph.edges.forEach((edge) => {
        if (edge.sections && edge.sections.length > 0) {
          const section = edge.sections[0];
          const points: ElkPoint[] = [
            section.startPoint,
            ...(section.bendPoints || []),
            section.endPoint,
          ];

          result.edges.push({
            id: edge.id,
            fromSocketId: edge.sources[0],
            toSocketId: edge.targets[0],
            points,
          });
        }
      });
    }

    return result;
  }

  /**
   * Find optimal position for a new node
   * @param components - Array of existing components
   * @param newComponentId - ID of the new component to position
   * @param parentId - Optional parent ID for hierarchical layouts
   */
  static async findOptimalNodePosition(
    components: (DiagramNodeData | DiagramGroupData)[],
    newComponentId: ComponentId,
    parentId?: ComponentId,
  ): Promise<{ x: number; y: number }> {
    // If no existing components, place at origin
    if (components.length === 0) {
      return { x: 100, y: 100 };
    }

    // Create a temporary component for the new node
    const existingNodes = [...components];
    const edges: DiagramEdgeData[] = [];

    // If we have a parent, place the node in the parent's subgraph
    if (parentId) {
      const parentNode = existingNodes.find((c) => c.def.id === parentId);
      if (parentNode) {
        // Filter to only include the parent and its children
        const subgraphComponents = existingNodes.filter(
          (c) => c.def.id === parentId || c.def.parentId === parentId,
        );

        // Add the new node to the subgraph
        const dummyNode: any = {
          def: {
            id: newComponentId,
            parentId,
            width: 200,
            height: 100,
          },
          sockets: [],
        };

        // Compute layout including the new node
        const result = await this.computeSubgraphLayout(
          parentId,
          [...subgraphComponents, dummyNode as DiagramNodeData],
          [],
        );

        // Find the new node in the result
        const newNodeLayout = result.nodes.find((n) => n.id === newComponentId);
        if (newNodeLayout) {
          return {
            x: newNodeLayout.position.x,
            y: newNodeLayout.position.y,
          };
        }
      }
    }

    // If no parent or parent not found, compute a good position in the main graph
    // Create a dummy node for the new component
    const dummyNode: any = {
      def: {
        id: newComponentId,
        width: 200,
        height: 100,
      },
      sockets: [],
    };

    // Add to existing nodes for layout
    const result = await this.computeLayout(
      [...existingNodes, dummyNode as DiagramNodeData],
      edges,
    );

    // Find the new node in the result
    const newNodeLayout = result.nodes.find((n) => n.id === newComponentId);
    if (newNodeLayout) {
      return {
        x: newNodeLayout.position.x,
        y: newNodeLayout.position.y,
      };
    }

    // Fallback if layout fails
    return { x: 100, y: 100 };
  }

  /**
   * Placeholder to generate a simple layout for testing
   * TODO: Remove once real ELK implementation is added
   */
  private static generatePlaceholderLayout(
    components: (DiagramNodeData | DiagramGroupData)[],
    edges: DiagramEdgeData[],
  ): LayoutResult {
    const nodes: LayoutNode[] = [];
    const layoutEdges: LayoutEdge[] = [];

    // Generate grid layout for nodes
    components.forEach((component, index) => {
      const col = index % 3;
      const row = Math.floor(index / 3);

      const width = 200;
      const height = 100;
      const x = col * 300 + 100;
      const y = row * 200 + 100;

      // Create sockets
      const sockets: LayoutSocket[] = component.sockets.map(
        (socket, socketIndex) => {
          const isLeft = socket.def.nodeSide === "left";
          return {
            id: socket.def.id,
            nodeSide: socket.def.nodeSide,
            position: {
              x: isLeft ? 0 : width,
              y: 30 + socketIndex * 20,
            },
          };
        },
      );

      nodes.push({
        id: component.def.id,
        parentId: component.def.parentId,
        position: { x, y, width, height },
        sockets,
      });
    });

    // Generate edges
    edges.forEach((edge) => {
      // Find source and target nodes
      const sourceNode = nodes.find((n) => n.id === edge.def.fromComponentId);
      const targetNode = nodes.find((n) => n.id === edge.def.toComponentId);

      if (!sourceNode || !targetNode) return;

      // Find source and target sockets
      const sourceSocket = sourceNode.sockets.find(
        (s) => s.id === edge.def.fromSocketId,
      );
      const targetSocket = targetNode.sockets.find(
        (s) => s.id === edge.def.toSocketId,
      );

      if (!sourceSocket || !targetSocket) return;

      // Calculate absolute socket positions
      const sourceX = sourceNode.position.x + sourceSocket.position.x;
      const sourceY = sourceNode.position.y + sourceSocket.position.y;
      const targetX = targetNode.position.x + targetSocket.position.x;
      const targetY = targetNode.position.y + targetSocket.position.y;

      // Create edge with simple routing
      layoutEdges.push({
        id: edge.def.id,
        fromSocketId: edge.def.fromSocketId,
        toSocketId: edge.def.toSocketId,
        points: [
          { x: sourceX, y: sourceY },
          { x: targetX, y: targetY },
        ],
      });
    });

    return { nodes, edges: layoutEdges };
  }
}
