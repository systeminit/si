/**
 * useLayoutEngine.ts
 *
 * A composable function that provides an interface to ELK layout engine.
 * This handles the creation and management of ELK graphs, layout computation,
 * and mapping between our application's types and ELK types.
 */

import { ref } from "vue";
import {
  DiagramNodeData,
  DiagramGroupData,
  DiagramEdgeData,
} from "@/components/ModelingDiagram/diagram_types";
import {
  ElkNode,
  ElkEdge,
  ElkPort,
  ElkPoint,
  ElkGraph,
  ElkLayoutOptions,
  LayoutResult,
  LayoutNode,
  LayoutEdge,
  LayoutSocket,
  ElkLayoutEngine,
} from "./ElkLayoutEngine";
import {
  LiveDiagramNode,
  LiveDiagramGroup,
  LiveDiagramEdge,
  LiveDiagramSocket,
  LiveDiagramElementType,
  LiveNodeLayoutConfig,
  LiveEdgeLayoutConfig,
  LiveSocketLayoutConfig,
  LiveLayoutConfig,
  LiveLayoutRequest,
} from "../live_diagram_types";

/**
 * Creates a layout engine service for managing ELK graph layout
 */
export function useLayoutEngine() {
  // Store references to all active graphs
  const graphs = ref<Record<string, ElkGraph>>({});

  // Track the main graph
  const mainGraphId = ref<string | null>(null);

  // Layout computation state
  const isComputing = ref(false);
  const lastLayoutResult = ref<LayoutResult | null>(null);
  const layoutError = ref<Error | null>(null);

  // Store references to original components and their data
  const componentReferences = {
    nodes: new Map<string, DiagramNodeData | DiagramGroupData>(),
    edges: new Map<string, DiagramEdgeData>(),
    originalComponents: new Map<string, LiveDiagramNode | LiveDiagramGroup>(),
    originalEdges: new Map<string, LiveDiagramEdge>(),
  };

  /**
   * Create a new graph with the given ID
   */
  function createGraph(graphId: string, options?: ElkLayoutOptions): ElkGraph {
    const graph: ElkGraph = {
      id: graphId,
      children: [],
      edges: [],
      layoutOptions: options || {
        algorithm: "layered",
        direction: "DOWN",
        spacing: 40,
        "elk.padding": "[40, 40, 40, 40]",
      },
    };

    // Initialize the graphs object if it doesn't exist
    if (!graphs.value) {
      graphs.value = {};
    }

    graphs.value[graphId] = graph;

    // If this is the first graph, set it as main
    if (mainGraphId.value === null) {
      mainGraphId.value = graphId;
    }

    return graph;
  }

  /**
   * Delete a graph by ID
   */
  function deleteGraph(graphId: string): void {
    if (graphs.value[graphId]) {
      delete graphs.value[graphId];

      // If we deleted the main graph, set a new one if available
      if (mainGraphId.value === graphId) {
        const graphIds = Object.keys(graphs.value);
        mainGraphId.value = graphIds[0] ?? null;
      }
    }
  }

  /**
   * Get a graph by ID
   */
  function getGraph(graphId: string): ElkGraph | undefined {
    return graphs.value[graphId];
  }

  /**
   * Add nodes to a graph
   */
  function addNodes(
    graphId: string,
    nodes: (LiveDiagramNode | LiveDiagramGroup)[],
  ): void {
    const graph = graphs.value[graphId];
    if (!graph) {
      throw new Error(`Graph '${graphId}' does not exist`);
    }

    // Convert nodes to ELK format and store original references by ID
    const elkNodes = nodes.map((node) => {
      // Store original component for later reference
      componentReferences.originalComponents.set(node.id, node);

      // Store original data if it exists
      if (node.originalData) {
        componentReferences.nodes.set(
          node.id,
          node.originalData as DiagramNodeData | DiagramGroupData,
        );
      } else {
        // Debug log to help identify issues
        console.warn(
          `Node ${node.id} (${node.type}) does not have originalData`,
        );
      }

      // If the node is in the nodeToElk map, store the mapping
      const elkNode = nodeToElk(node);
      return elkNode;
    });

    // Add nodes to graph - ensure we replace all existing nodes
    graph.children = [...elkNodes];
  }

  /**
   * Add edges to a graph
   */
  function addEdges(graphId: string, edges: LiveDiagramEdge[]): void {
    const graph = graphs.value[graphId];
    if (!graph) {
      throw new Error(`Graph '${graphId}' does not exist`);
    }

    // Convert edges to ELK format and store original references by ID
    const elkEdges = edges.map((edge) => {
      // Store original edge for later reference
      componentReferences.originalEdges.set(edge.id, edge);

      // Store original data if it exists
      if (edge.originalData) {
        componentReferences.edges.set(
          edge.id,
          edge.originalData as DiagramEdgeData,
        );
      } else {
        // Debug log to help identify issues
        console.warn(`Edge ${edge.id} does not have originalData`);
      }

      return edgeToElk(edge);
    });

    // Add edges to graph - replace all existing edges
    graph.edges = [...elkEdges];
  }

  /**
   * Clear all nodes and edges from a graph
   */
  function clearGraph(graphId: string): void {
    const graph = graphs.value[graphId];
    if (!graph) {
      throw new Error(`Graph '${graphId}' does not exist`);
    }

    // Get IDs of all nodes and edges in this graph
    const nodeIds = graph.children.map((node) => node.id);
    const edgeIds = graph.edges.map((edge) => edge.id);

    // Clear references to these nodes and edges
    nodeIds.forEach((id) => {
      componentReferences.originalComponents.delete(id);
      componentReferences.nodes.delete(id);
    });

    edgeIds.forEach((id) => {
      componentReferences.originalEdges.delete(id);
      componentReferences.edges.delete(id);
    });

    // Clear the graph
    graph.children = [];
    graph.edges = [];
  }

  /**
   * Update layout options for a graph
   */
  function updateLayoutOptions(
    graphId: string,
    options: ElkLayoutOptions,
  ): void {
    const graph = graphs.value[graphId];
    if (!graph) {
      throw new Error(`Graph '${graphId}' does not exist`);
    }

    graph.layoutOptions = {
      ...(graph.layoutOptions || {}),
      ...options,
    };
  }

  /**
   * Compute layout for a graph
   */
  async function computeLayout(
    graphId: string = mainGraphId.value || "main",
  ): Promise<LayoutResult> {
    const graph = graphs.value[graphId];
    if (!graph) {
      throw new Error(`Graph '${graphId}' does not exist`);
    }

    isComputing.value = true;
    layoutError.value = null;

    try {
      // If we don't have any nodes in the graph, we can't compute a layout
      if (graph.children.length === 0) {
        console.warn("No nodes in graph, skipping layout");
        return { nodes: [], edges: [] };
      }

      // Use our stored references to original components instead of converting types
      const nodesFromGraph: (DiagramNodeData | DiagramGroupData)[] = [];
      const edgesFromGraph: DiagramEdgeData[] = [];

      // Collect original component data from our reference maps
      for (const node of graph.children) {
        const originalData = componentReferences.nodes.get(node.id);
        if (originalData) {
          nodesFromGraph.push(originalData);
        } else {
          console.warn(`No original data found for node ${node.id}`);

          // If we don't have original data, create minimal data from the ElkNode
          // This allows layout to still work even if references are missing
          const minimalData = {
            def: {
              id: node.id,
              width: node.width,
              height: node.height,
              x: node.x || 0,
              y: node.y || 0,
              isGroup: false,
            },
            sockets:
              node.ports?.map((port) => ({
                def: {
                  id: port.id,
                  nodeSide: (port.properties?.side === "WEST"
                    ? "left"
                    : port.properties?.side === "EAST"
                    ? "right"
                    : port.properties?.side === "NORTH"
                    ? "top"
                    : "bottom") as any,
                },
              })) || [],
            uniqueKey: node.id,
          };
          nodesFromGraph.push(minimalData as any);
        }
      }

      for (const edge of graph.edges) {
        const originalData = componentReferences.edges.get(edge.id);
        if (originalData) {
          edgesFromGraph.push(originalData);
        } else {
          console.warn(`No original data found for edge ${edge.id}`);

          // Create minimal edge data if original not found
          const minimalData = {
            def: {
              id: edge.id,
              fromSocketId: edge.sources[0],
              toSocketId: edge.targets[0],
              // Attempt to derive component IDs from socket IDs if possible
              fromComponentId: "",
              toComponentId: "",
            },
            uniqueKey: edge.id,
          };
          edgesFromGraph.push(minimalData as any);
        }
      }

      // Provide detailed debug info if we're missing data
      if (nodesFromGraph.length === 0 && graph.children.length > 0) {
        console.warn(
          "No original component data found for layout calculation",
          {
            graphNodeIds: graph.children.map((n) => n.id),
            referenceNodeIds: Array.from(componentReferences.nodes.keys()),
            graphNodeCount: graph.children.length,
            referenceNodeCount: componentReferences.nodes.size,
          },
        );
      }

      // Call the ElkLayoutEngine class to compute the layout
      const result = await ElkLayoutEngine.computeLayout(
        nodesFromGraph,
        edgesFromGraph,
        graph.layoutOptions as ElkLayoutOptions,
      );

      lastLayoutResult.value = result;
      return result;
    } catch (error) {
      layoutError.value = error as Error;
      throw error;
    } finally {
      isComputing.value = false;
    }
  }

  /**
   * Compute optimal position for a new node
   */
  async function findOptimalNodePosition(
    graphId: string,
    node: LiveDiagramNode | LiveDiagramGroup,
    nearNodeId?: string,
  ): Promise<{ x: number; y: number }> {
    const graph = graphs.value[graphId];
    if (!graph) {
      throw new Error(`Graph '${graphId}' does not exist`);
    }

    // Get existing nodes from the graph using our stored references
    const existingNodes: (DiagramNodeData | DiagramGroupData)[] = [];

    // Use our stored references to original components to get existing nodes
    for (const existingNode of graph.children) {
      if (existingNode.id !== node.id) {
        // Exclude the new node if it's already in the graph
        const originalData = componentReferences.nodes.get(existingNode.id);
        if (originalData) {
          existingNodes.push(originalData);
        }
      }
    }

    // Store the new node's data
    componentReferences.originalComponents.set(node.id, node);
    if (node.originalData) {
      componentReferences.nodes.set(
        node.id,
        node.originalData as DiagramNodeData | DiagramGroupData,
      );
    }

    // Use the ElkLayoutEngine to find an optimal position
    return ElkLayoutEngine.findOptimalNodePosition(
      existingNodes as DiagramNodeData[],
      node.id,
      nearNodeId,
    );
  }

  /**
   * Create a complete layout request for a set of elements
   */
  function createLayoutRequest(
    nodes: (LiveDiagramNode | LiveDiagramGroup)[],
    edges: LiveDiagramEdge[],
    sockets: LiveDiagramSocket[],
    config: LiveLayoutConfig,
  ): LiveLayoutRequest {
    // Create node configs
    const nodeConfigs: LiveNodeLayoutConfig[] = nodes.map((node) => ({
      id: node.id,
      width: node.position.width,
      height: node.position.height,
      x: node.position.x,
      y: node.position.y,
      parentId: "parentId" in node ? node.parentId : undefined,
      type: node.type === LiveDiagramElementType.GROUP ? "group" : "node",
    }));

    // Create socket configs
    const socketConfigs: LiveSocketLayoutConfig[] = sockets.map((socket) => ({
      id: socket.id,
      parentId: socket.parentId,
      side: socket.side,
    }));

    // Create edge configs
    const edgeConfigs: LiveEdgeLayoutConfig[] = edges.map((edge) => ({
      id: edge.id,
      fromSocketId: edge.fromSocketId,
      toSocketId: edge.toSocketId,
      fromNodeId: edge.fromNodeId,
      toNodeId: edge.toNodeId,
      type: edge.isManagement ? "management" : "standard",
    }));

    return {
      nodes: nodeConfigs,
      edges: edgeConfigs,
      sockets: socketConfigs,
      config,
    };
  }

  // Conversion functions

  /**
   * Convert LiveDiagramNode to ElkNode
   */
  function nodeToElk(node: LiveDiagramNode | LiveDiagramGroup): ElkNode {
    // Create sockets as ports
    const ports: ElkPort[] = node.sockets.map((socket) => ({
      id: socket.id,
      width: 10,
      height: 10,
      properties: {
        side: socketSideToElk(socket.side),
      },
    }));

    // Create ElkNode
    return {
      id: node.id,
      width: node.position.width,
      height: node.position.height,
      x: node.position.x,
      y: node.position.y,
      ports,
    };
  }

  /**
   * Convert LiveDiagramEdge to ElkEdge
   */
  function edgeToElk(edge: LiveDiagramEdge): ElkEdge {
    return {
      id: edge.id,
      sources: [edge.fromSocketId],
      targets: [edge.toSocketId],
    };
  }

  /**
   * Convert socket side to ELK side
   */
  function socketSideToElk(
    side: "left" | "right" | "top" | "bottom",
  ): "WEST" | "EAST" | "NORTH" | "SOUTH" {
    switch (side) {
      case "left":
        return "WEST";
      case "right":
        return "EAST";
      case "top":
        return "NORTH";
      case "bottom":
        return "SOUTH";
      default:
        return "EAST"; // Default fallback, though type system should prevent this
    }
  }

  /**
   * Convert ELK side to socket side
   */
  function elkSideToSocket(
    side: "WEST" | "EAST" | "NORTH" | "SOUTH",
  ): "left" | "right" | "top" | "bottom" {
    switch (side) {
      case "WEST":
        return "left";
      case "EAST":
        return "right";
      case "NORTH":
        return "top";
      case "SOUTH":
        return "bottom";
      default:
        return "right"; // Default fallback, though type system should prevent this
    }
  }

  /**
   * Convert ELK layout result to our format
   */
  function elkToLayoutResult(elkGraph: ElkGraph): LayoutResult {
    const nodes: LayoutNode[] = [];
    const edges: LayoutEdge[] = [];

    // Process nodes
    elkGraph.children.forEach((elkNode) => {
      const sockets: LayoutSocket[] = [];

      // Process ports (sockets)
      elkNode.ports?.forEach((port) => {
        if (!port.x || !port.y || !port.properties?.side) return;

        sockets.push({
          id: port.id,
          position: {
            x: port.x,
            y: port.y,
          },
          nodeSide: elkSideToSocket(
            port.properties.side as "WEST" | "EAST" | "NORTH" | "SOUTH",
          ),
        });
      });

      // Add node with its sockets
      nodes.push({
        id: elkNode.id,
        position: {
          x: elkNode.x || 0,
          y: elkNode.y || 0,
          width: elkNode.width,
          height: elkNode.height,
        },
        sockets,
      });
    });

    // Process edges
    elkGraph.edges.forEach((elkEdge) => {
      if (
        !elkEdge.sections ||
        elkEdge.sections.length === 0 ||
        !elkEdge.sections[0] ||
        !elkEdge.sources[0] ||
        !elkEdge.targets[0]
      )
        return;

      const section = elkEdge.sections[0];
      const points: ElkPoint[] = [
        section.startPoint,
        ...(section.bendPoints || []),
        section.endPoint,
      ];

      edges.push({
        id: elkEdge.id,
        fromSocketId: elkEdge.sources[0],
        toSocketId: elkEdge.targets[0],
        points,
      });
    });

    return { nodes, edges };
  }

  /**
   * Since we're now using the actual ElkLayoutEngine for layout computation,
   * we don't need the placeholder layout generator anymore.
   */

  // Return the layout engine API
  return {
    // State
    graphs,
    mainGraphId,
    isComputing,
    lastLayoutResult,
    layoutError,
    componentReferences,

    // Graph management
    createGraph,
    deleteGraph,
    getGraph,
    addNodes,
    addEdges,
    clearGraph,
    updateLayoutOptions,

    // Layout computation
    computeLayout,
    findOptimalNodePosition,
    createLayoutRequest,

    // Conversion utilities
    nodeToElk,
    edgeToElk,
    elkToLayoutResult,
  };
}

export default useLayoutEngine;
