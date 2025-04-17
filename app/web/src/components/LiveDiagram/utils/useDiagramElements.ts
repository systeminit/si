/**
 * useDiagramElements.ts
 *
 * A composable function that manages diagram elements (nodes, edges, sockets)
 * for the LiveDiagram component. It provides reactivity, registration, and retrieval
 * of diagram elements, as well as selection and hover state management.
 */

import { ref, computed, reactive, toRefs } from "vue";
import { IRect, Vector2d } from "konva/lib/types";
import {
  DiagramNodeData,
  DiagramGroupData,
  DiagramEdgeData,
} from "@/components/ModelingDiagram/diagram_types";
import {
  LiveDiagramNode,
  LiveDiagramGroup,
  LiveDiagramEdge,
  LiveDiagramSocket,
  LiveDiagramElementType,
  LiveDiagramAnyElement,
  LiveDiagramFactory,
} from "../live_diagram_types";

/**
 * Creates a service for managing diagram elements
 */
export function useDiagramElements() {
  // Element registries
  const nodes = ref<Record<string, LiveDiagramNode>>({});
  const groups = ref<Record<string, LiveDiagramGroup>>({});
  const edges = ref<Record<string, LiveDiagramEdge>>({});
  const sockets = ref<Record<string, LiveDiagramSocket>>({});

  // Selected state
  const selectedElementIds = ref<string[]>([]);
  const hoveredElementId = ref<string | null>(null);

  // Computed collections
  const allElements = computed<Record<string, LiveDiagramAnyElement>>(() => {
    return {
      ...nodes.value,
      ...groups.value,
      ...edges.value,
      ...sockets.value,
    };
  });

  // Computed selected elements
  const selectedElements = computed(() => {
    return selectedElementIds.value
      .map((id) => allElements.value[id])
      .filter(Boolean);
  });

  // Computed hovered element
  const hoveredElement = computed(() => {
    return hoveredElementId.value
      ? allElements.value[hoveredElementId.value]
      : null;
  });

  /**
   * Register a node in the diagram
   */
  function registerNode(node: LiveDiagramNode): void {
    nodes.value[node.id] = node;
  }

  /**
   * Register a group in the diagram
   */
  function registerGroup(group: LiveDiagramGroup): void {
    groups.value[group.id] = group;
  }

  /**
   * Register an edge in the diagram
   */
  function registerEdge(edge: LiveDiagramEdge): void {
    edges.value[edge.id] = edge;
  }

  /**
   * Register a socket in the diagram
   */
  function registerSocket(socket: LiveDiagramSocket): void {
    sockets.value[socket.id] = socket;
  }

  /**
   * Get an element by ID
   */
  function getElementById(id: string): LiveDiagramAnyElement | undefined {
    return allElements.value[id];
  }

  /**
   * Get elements by type
   */
  function getElementsByType(
    type: LiveDiagramElementType,
  ): LiveDiagramAnyElement[] {
    return Object.values(allElements.value).filter((el) => el.type === type);
  }

  /**
   * Get all nodes (including groups)
   */
  function getAllNodes(): (LiveDiagramNode | LiveDiagramGroup)[] {
    return [...Object.values(nodes.value), ...Object.values(groups.value)];
  }

  /**
   * Get all container nodes (nodes or groups)
   */
  function getContainers(): (LiveDiagramNode | LiveDiagramGroup)[] {
    return getAllNodes();
  }

  /**
   * Select an element
   */
  function selectElement(id: string, addToSelection = false): void {
    if (addToSelection) {
      if (!selectedElementIds.value.includes(id)) {
        selectedElementIds.value = [...selectedElementIds.value, id];
      }
    } else {
      selectedElementIds.value = [id];
    }
  }

  /**
   * Deselect an element
   */
  function deselectElement(id: string): void {
    selectedElementIds.value = selectedElementIds.value.filter(
      (eId) => eId !== id,
    );
  }

  /**
   * Select multiple elements
   */
  function selectElements(ids: string[]): void {
    selectedElementIds.value = ids;
  }

  /**
   * Clear all selections
   */
  function clearSelection(): void {
    selectedElementIds.value = [];
  }

  /**
   * Check if an element is selected
   */
  function isElementSelected(element: LiveDiagramAnyElement | string): boolean {
    const id = typeof element === "string" ? element : element.id;
    return selectedElementIds.value.includes(id);
  }

  /**
   * Set hovered element
   */
  function setHoveredElement(id: string | null): void {
    hoveredElementId.value = id;
  }

  /**
   * Check if an element is hovered
   */
  function isElementHovered(element: LiveDiagramAnyElement | string): boolean {
    const id = typeof element === "string" ? element : element.id;
    return hoveredElementId.value === id;
  }

  /**
   * Convert store data to LiveDiagram elements
   */
  function registerComponentFromStore(
    component: DiagramNodeData | DiagramGroupData,
    pos: IRect,
  ): LiveDiagramNode | LiveDiagramGroup {
    const position = {
      x: pos.x,
      y: pos.y,
      width: pos.width,
      height: pos.height,
    };

    // Create node or group based on component type
    const element = component.def.isGroup
      ? LiveDiagramFactory.createGroup(component as DiagramGroupData, position)
      : LiveDiagramFactory.createNode(component as DiagramNodeData, position);

    // Register sockets
    component.sockets.forEach((socketData) => {
      const socket = LiveDiagramFactory.createSocket(socketData, element.id);
      registerSocket(socket);

      // Add socket to node/group
      element.sockets.push(socket);
    });

    // Register the element
    if (element.type === LiveDiagramElementType.GROUP) {
      registerGroup(element as LiveDiagramGroup);
    } else {
      registerNode(element as LiveDiagramNode);
    }

    return element;
  }

  /**
   * Register an edge from store data
   */
  function registerEdgeFromStore(edgeData: DiagramEdgeData): LiveDiagramEdge {
    // Create the edge
    const edge = LiveDiagramFactory.createEdge(edgeData);

    // Get source and target sockets
    const fromSocket = sockets.value[edge.fromSocketId];
    const toSocket = sockets.value[edge.toSocketId];

    if (fromSocket && toSocket) {
      // Get the parent nodes of the sockets
      const fromNode =
        nodes.value[fromSocket.parentId] || groups.value[fromSocket.parentId];
      const toNode =
        nodes.value[toSocket.parentId] || groups.value[toSocket.parentId];

      if (fromNode && toNode) {
        // Calculate absolute socket positions
        const fromPos = {
          x: fromNode.position.x + fromSocket.position.x,
          y: fromNode.position.y + fromSocket.position.y,
        };

        const toPos = {
          x: toNode.position.x + toSocket.position.x,
          y: toNode.position.y + toSocket.position.y,
        };

        // Update edge with correct points
        edge.points = [fromPos, toPos];

        console.log(`Edge ${edge.id} created with points:`, edge.points);
      } else {
        console.warn(
          `Missing node for edge ${
            edge.id
          }: fromNode=${!!fromNode}, toNode=${!!toNode}`,
        );
      }
    } else {
      console.warn(
        `Missing socket for edge ${
          edge.id
        }: fromSocket=${!!fromSocket}, toSocket=${!!toSocket}`,
      );
    }

    registerEdge(edge);
    return edge;
  }

  /**
   * Update element position
   */
  function updateElementPosition(
    element: LiveDiagramNode | LiveDiagramGroup,
    position: Partial<Vector2d & { width?: number; height?: number }>,
  ): void {
    const elementToUpdate =
      element.type === LiveDiagramElementType.GROUP
        ? groups.value[element.id]
        : nodes.value[element.id];

    if (!elementToUpdate) {
      console.error(
        `Element with id ${element.id} not found for position update`,
      );
      return;
    }

    // Log before update
    console.log(`Updating position for ${element.id}`, {
      before: { ...elementToUpdate.position },
      update: { ...position },
    });

    // Make sure positions are numbers
    const numX =
      typeof position.x === "number"
        ? position.x
        : parseFloat(String(position.x));
    const numY =
      typeof position.y === "number"
        ? position.y
        : parseFloat(String(position.y));
    const numWidth =
      typeof position.width === "number"
        ? position.width
        : parseFloat(String(position.width));
    const numHeight =
      typeof position.height === "number"
        ? position.height
        : parseFloat(String(position.height));

    // Update position properties
    if (!isNaN(numX)) elementToUpdate.position.x = numX;
    if (!isNaN(numY)) elementToUpdate.position.y = numY;
    if (!isNaN(numWidth)) elementToUpdate.position.width = numWidth;
    if (!isNaN(numHeight)) elementToUpdate.position.height = numHeight;

    // Log after update to verify changes were applied
    console.log(`Position after update for ${element.id}:`, {
      after: { ...elementToUpdate.position },
    });

    // Update socket positions based on the new container position
    elementToUpdate.sockets.forEach((socketId) => {
      const socket =
        sockets.value[typeof socketId === "string" ? socketId : socketId.id];
      if (socket) {
        updateSocketPosition(socket);
      }
    });

    // Update connected edges to reflect the new node position
    updateConnectedEdges(elementToUpdate.id);
  }

  /**
   * Update edges connected to a node
   */
  function updateConnectedEdges(nodeId: string): void {
    // Find all edges connected to this node
    const connectedEdges = Object.values(edges.value).filter(
      (edge) => edge.fromNodeId === nodeId || edge.toNodeId === nodeId,
    );

    if (connectedEdges.length === 0) return;

    console.log(
      `Updating ${connectedEdges.length} edges connected to node ${nodeId}`,
    );

    // Update each connected edge
    connectedEdges.forEach((edge) => {
      // Get source and target sockets
      const fromSocket = sockets.value[edge.fromSocketId];
      const toSocket = sockets.value[edge.toSocketId];

      if (!fromSocket || !toSocket) {
        console.warn(`Missing socket for edge ${edge.id}`);
        return;
      }

      // Get absolute positions of both sockets
      const fromNode =
        nodes.value[fromSocket.parentId] || groups.value[fromSocket.parentId];
      const toNode =
        nodes.value[toSocket.parentId] || groups.value[toSocket.parentId];

      if (!fromNode || !toNode) {
        console.warn(`Missing node for edge ${edge.id}`);
        return;
      }

      // Calculate absolute socket positions
      const fromPos = {
        x: fromNode.position.x + fromSocket.position.x,
        y: fromNode.position.y + fromSocket.position.y,
      };

      const toPos = {
        x: toNode.position.x + toSocket.position.x,
        y: toNode.position.y + toSocket.position.y,
      };

      // Create a simple straight line between sockets
      const updatedPoints = [fromPos, toPos];

      // Apply the updated points to the edge
      edge.points = updatedPoints;

      console.log(`Updated edge ${edge.id} points:`, updatedPoints);
    });
  }

  /**
   * Update socket position based on its parent and side
   */
  function updateSocketPosition(socket: LiveDiagramSocket): void {
    const parentNode =
      nodes.value[socket.parentId] || groups.value[socket.parentId];
    if (!parentNode) {
      console.warn(
        `Socket ${socket.id} parent node ${socket.parentId} not found`,
      );
      return;
    }

    const { width, height } = parentNode.position;
    let x = 0;
    let y = 0;

    // Get all sockets on the same parent and side
    const sameParentSideSockets = Object.values(sockets.value).filter(
      (s) => s.parentId === socket.parentId && s.side === socket.side,
    );

    const socketCount = sameParentSideSockets.length;
    const socketIndex = sameParentSideSockets.findIndex(
      (s) => s.id === socket.id,
    );

    // Calculate position based on side and index
    switch (socket.side) {
      case "left":
        x = 0;
        if (socketCount > 1) {
          // Distribute evenly along the left side
          const segmentHeight = height / (socketCount + 1);
          y = segmentHeight * (socketIndex + 1);
        } else {
          // Single socket, place in the middle
          y = height / 2;
        }
        break;
      case "right":
        x = width;
        if (socketCount > 1) {
          // Distribute evenly along the right side
          const segmentHeight = height / (socketCount + 1);
          y = segmentHeight * (socketIndex + 1);
        } else {
          // Single socket, place in the middle
          y = height / 2;
        }
        break;
      case "top":
        y = 0;
        if (socketCount > 1) {
          // Distribute evenly along the top side
          const segmentWidth = width / (socketCount + 1);
          x = segmentWidth * (socketIndex + 1);
        } else {
          // Single socket, place in the middle
          x = width / 2;
        }
        break;
      case "bottom":
        y = height;
        if (socketCount > 1) {
          // Distribute evenly along the bottom side
          const segmentWidth = width / (socketCount + 1);
          x = segmentWidth * (socketIndex + 1);
        } else {
          // Single socket, place in the middle
          x = width / 2;
        }
        break;
      default:
        // Fallback to middle of node
        x = width / 2;
        y = height / 2;
        break;
    }

    // Update socket position
    socket.position = { x, y };

    if (socketIndex === 0) {
      // Log once per side to avoid console spam
      console.log(
        `Positioned ${socketCount} sockets on ${socket.side} side of ${socket.parentId}`,
      );
    }
  }

  /**
   * Update edge points
   */
  function updateEdgePoints(edge: LiveDiagramEdge, points: Vector2d[]): void {
    const edgeToUpdate = edges.value[edge.id];
    if (!edgeToUpdate) return;

    edgeToUpdate.points = points;
  }

  /**
   * Clear all elements
   */
  function clearElements(): void {
    // Clear all registries
    nodes.value = {};
    groups.value = {};
    edges.value = {};
    sockets.value = {};

    // Clear selection
    clearSelection();
    setHoveredElement(null);
  }

  // Return the diagram elements API
  return {
    // Element registries
    nodes,
    groups,
    edges,
    sockets,
    allElements,

    // Selection state
    selectedElementIds,
    selectedElements,
    hoveredElementId,
    hoveredElement,

    // Registration methods
    registerNode,
    registerGroup,
    registerEdge,
    registerSocket,
    registerComponentFromStore,
    registerEdgeFromStore,

    // Query methods
    getElementById,
    getElementsByType,
    getAllNodes,
    getContainers,

    // Selection methods
    selectElement,
    deselectElement,
    selectElements,
    clearSelection,
    isElementSelected,

    // Hover methods
    setHoveredElement,
    isElementHovered,

    // Update methods
    updateElementPosition,
    updateSocketPosition,
    updateEdgePoints,
    updateConnectedEdges,

    // Cleanup method
    clearElements,
  };
}

export default useDiagramElements;
