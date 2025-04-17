/**
 * useDiagramInteractions.ts
 *
 * A composable function that manages interactions with the diagram,
 * including dragging, selecting, connecting, and zooming operations.
 */

import { ref, computed, reactive, toRefs } from "vue";
import { Vector2d } from "konva/lib/types";
import {
  LiveDiagramSocket,
  LiveDiagramEvent,
  LiveDiagramDragEvent,
  LiveDiagramConnectEvent,
} from "../live_diagram_types";

/**
 * Creates an interaction manager for the diagram
 */
export function useDiagramInteractions() {
  // Interaction state
  const isDragging = ref(false);
  const dragStartPosition = ref<Vector2d | null>(null);
  const currentDragPosition = ref<Vector2d | null>(null);
  const draggedElements = ref<string[]>([]);
  const draggedEdges = ref<{ id: string; points: Vector2d[] }[]>([]);

  // Selection box (rectangular selection)
  const selectionBoxActive = ref(false);
  const selectionBoxStart = ref<Vector2d | null>(null);
  const selectionBoxEnd = ref<Vector2d | null>(null);

  // Edge creation
  const isCreatingEdge = ref(false);
  const edgeStartSocket = ref<LiveDiagramSocket | null>(null);
  const edgeEndPosition = ref<Vector2d | null>(null);

  // Pan and zoom
  const pan = ref<Vector2d | null>(null);
  const x = ref(0);
  const y = ref(0);
  const zoomLevel = ref(1);

  // Cursor and mode
  const cursor = ref("default");
  const interactionMode = ref<"select" | "pan" | "connect">("select");

  // Events
  const lastEvent = ref<LiveDiagramEvent | null>(null);

  // Computed values for current selection box bounds
  const selectionBoxBounds = computed(() => {
    if (!selectionBoxStart.value || !selectionBoxEnd.value) return null;

    const x1 = selectionBoxStart.value.x;
    const y1 = selectionBoxStart.value.y;
    const x2 = selectionBoxEnd.value.x;
    const y2 = selectionBoxEnd.value.y;

    return {
      x: Math.min(x1, x2),
      y: Math.min(y1, y2),
      width: Math.abs(x2 - x1),
      height: Math.abs(y2 - y1),
    };
  });

  // Computed values for total drag delta
  const dragDelta = computed(() => {
    if (!dragStartPosition.value || !currentDragPosition.value)
      return { x: 0, y: 0 };

    return {
      x: currentDragPosition.value.x - dragStartPosition.value.x,
      y: currentDragPosition.value.y - dragStartPosition.value.y,
    };
  });

  /**
   * Start dragging elements
   */
  function startDrag(position: Vector2d, elementIds: string[]): void {
    isDragging.value = true;
    dragStartPosition.value = { ...position };
    currentDragPosition.value = { ...position };
    draggedElements.value = [...elementIds];
    draggedEdges.value = []; // Clear any previous dragged edges

    cursor.value = "grabbing";

    // Create drag start event
    lastEvent.value = {
      type: "dragStart",
      element: {
        id: elementIds[0],
        type: "node",
      } as LiveDiagramEvent["element"], // Simplified element reference
      position: { ...position },
    } as LiveDiagramDragEvent;
  }

  /**
   * Update drag position
   */
  function updateDrag(position: Vector2d): void {
    if (!isDragging.value) return;

    currentDragPosition.value = { ...position };

    // Create drag move event
    lastEvent.value = {
      type: "dragMove",
      element: {
        id: draggedElements.value[0],
        type: "node",
      } as LiveDiagramEvent["element"], // Simplified element reference
      position: { ...position },
      delta: {
        x: position.x - (dragStartPosition.value?.x || 0),
        y: position.y - (dragStartPosition.value?.y || 0),
      },
    } as LiveDiagramDragEvent;
  }

  /**
   * Update dragged edges
   * This calculates temporary edge positions during dragging
   */
  function updateDraggedEdges(
    nodeId: string,
    allNodes: Record<string, any>,
    allGroups: Record<string, any>,
    allSockets: Record<string, any>,
    allEdges: Record<string, any>,
    delta: Vector2d,
  ): void {
    // Find all edges connected to this node
    const connectedEdges = Object.values(allEdges).filter(
      (edge: any) => edge.fromNodeId === nodeId || edge.toNodeId === nodeId,
    );

    if (connectedEdges.length === 0) return;

    const newDraggedEdges: { id: string; points: Vector2d[] }[] = [];

    // Update each connected edge
    connectedEdges.forEach((edge: any) => {
      // Get source and target sockets
      const fromSocket = allSockets[edge.fromSocketId];
      const toSocket = allSockets[edge.toSocketId];

      if (!fromSocket || !toSocket) return;

      // Get the nodes
      const fromNode =
        allNodes[fromSocket.parentId] || allGroups[fromSocket.parentId];
      const toNode =
        allNodes[toSocket.parentId] || allGroups[toSocket.parentId];

      if (!fromNode || !toNode) return;

      // Calculate absolute socket positions with potential offset for dragged node
      const fromPos = {
        x:
          fromNode.position.x +
          (fromNode.id === nodeId ? delta.x : 0) +
          fromSocket.position.x,
        y:
          fromNode.position.y +
          (fromNode.id === nodeId ? delta.y : 0) +
          fromSocket.position.y,
      };

      const toPos = {
        x:
          toNode.position.x +
          (toNode.id === nodeId ? delta.x : 0) +
          toSocket.position.x,
        y:
          toNode.position.y +
          (toNode.id === nodeId ? delta.y : 0) +
          toSocket.position.y,
      };

      // Push to dragged edges
      newDraggedEdges.push({
        id: edge.id,
        points: [fromPos, toPos],
      });
    });

    // Update the dragged edges
    draggedEdges.value = newDraggedEdges;
  }

  /**
   * End dragging elements
   */
  function endDrag(): void {
    if (!isDragging.value) return;

    // Create drag end event
    lastEvent.value = {
      type: "dragEnd",
      element: {
        id: draggedElements.value[0],
        type: "node",
      } as LiveDiagramEvent["element"], // Simplified element reference
      position: { ...(currentDragPosition.value || { x: 0, y: 0 }) },
      delta: dragDelta.value,
    } as LiveDiagramDragEvent;

    // Reset drag state
    isDragging.value = false;
    dragStartPosition.value = null;
    currentDragPosition.value = null;
    draggedElements.value = [];
    draggedEdges.value = []; // Clear dragged edges

    cursor.value = "default";
  }

  /**
   * Start selection box operation
   */
  function startSelectionBox(position: Vector2d): void {
    selectionBoxActive.value = true;
    selectionBoxStart.value = { ...position };
    selectionBoxEnd.value = { ...position };
  }

  /**
   * Update selection box
   */
  function updateSelectionBox(position: Vector2d): void {
    if (!selectionBoxActive.value) return;

    selectionBoxEnd.value = { ...position };
  }

  /**
   * End selection box operation
   */
  function endSelectionBox(): {
    x: number;
    y: number;
    width: number;
    height: number;
  } | null {
    if (!selectionBoxActive.value || !selectionBoxBounds.value) {
      selectionBoxActive.value = false;
      selectionBoxStart.value = null;
      selectionBoxEnd.value = null;
      return null;
    }

    const bounds = { ...selectionBoxBounds.value };

    // Reset selection box state
    selectionBoxActive.value = false;
    selectionBoxStart.value = null;
    selectionBoxEnd.value = null;

    return bounds;
  }

  /**
   * Start creating an edge
   */
  function startEdgeCreation(
    socket: LiveDiagramSocket,
    position: Vector2d,
  ): void {
    isCreatingEdge.value = true;
    edgeStartSocket.value = socket;
    edgeEndPosition.value = { ...position };

    cursor.value = "crosshair";
  }

  /**
   * Update edge creation
   */
  function updateEdgeCreation(position: Vector2d): void {
    if (!isCreatingEdge.value) return;

    edgeEndPosition.value = { ...position };
  }

  /**
   * End edge creation
   */
  function endEdgeCreation(targetSocket?: LiveDiagramSocket): void {
    if (!isCreatingEdge.value || !edgeStartSocket.value) {
      // Reset edge creation state
      isCreatingEdge.value = false;
      edgeStartSocket.value = null;
      edgeEndPosition.value = null;
      cursor.value = "default";
      return;
    }

    const sourceSocket = edgeStartSocket.value;

    // Create edge connection event
    lastEvent.value = {
      type: "connect",
      element: { id: "temp-edge", type: "edge" } as LiveDiagramEvent["element"], // Simplified element reference
      sourceSocket,
      targetSocket,
    } as LiveDiagramConnectEvent;

    // Reset edge creation state
    isCreatingEdge.value = false;
    edgeStartSocket.value = null;
    edgeEndPosition.value = null;

    cursor.value = "default";
  }

  /**
   * Update pan offset
   */
  function updatePan(newX: number, newY: number): void {
    // Update pan values - for Konva, positive values move content right/down
    // So we invert the values here to match the natural panning convention
    // where dragging right moves the view right (content left)
    x.value = -newX;
    y.value = -newY;
  }

  /**
   * Set zoom level
   */
  function setZoom(newZoomLevel: number): void {
    // Constrain zoom level between MIN_ZOOM and MAX_ZOOM
    if (newZoomLevel < MIN_ZOOM) zoomLevel.value = MIN_ZOOM;
    else if (newZoomLevel > MAX_ZOOM) zoomLevel.value = MAX_ZOOM;
    else zoomLevel.value = newZoomLevel;

    // Store zoom level in localStorage for persistence between sessions
    if (zoomLevel.value === 1) {
      window.localStorage.removeItem("si-diagram-zoom");
    } else {
      window.localStorage.setItem("si-diagram-zoom", `${zoomLevel.value}`);
    }
  }

  // Constants for zoom limits - match the ones used in ModelingDiagram
  const MIN_ZOOM = 0.1; // 10%
  const MAX_ZOOM = 10; // 1000%
  const ZOOM_PAN_FACTOR = 1.0; // Pan speed factor

  /**
   * Zoom in (by a step amount)
   */
  function zoomIn(step = 0.1): void {
    setZoom(zoomLevel.value + step);
  }

  /**
   * Zoom out (by a step amount)
   */
  function zoomOut(step = 0.1): void {
    setZoom(zoomLevel.value - step);
  }

  /**
   * Reset zoom to default
   */
  function resetZoom(): void {
    zoomLevel.value = 1;
  }

  /**
   * Reset pan to default
   */
  function resetPan(): void {
    x.value = 0;
    y.value = 0;
  }

  /**
   * Set interaction mode
   */
  function setInteractionMode(mode: "select" | "pan" | "connect"): void {
    interactionMode.value = mode;

    // Update cursor based on mode
    switch (mode) {
      case "select":
        cursor.value = "default";
        break;
      case "pan":
        cursor.value = "grab";
        break;
      case "connect":
        cursor.value = "crosshair";
        break;
      default:
        cursor.value = "default";
        break;
    }
  }

  /**
   * Set cursor
   */
  function setCursor(newCursor: string): void {
    cursor.value = newCursor;
  }

  /**
   * Check if an element is within the selection box
   */
  function isInSelectionBox(
    position: Vector2d,
    width: number,
    height: number,
  ): boolean {
    if (!selectionBoxBounds.value) return false;

    const {
      x,
      y,
      width: boxWidth,
      height: boxHeight,
    } = selectionBoxBounds.value;

    return (
      position.x < x + boxWidth &&
      position.x + width > x &&
      position.y < y + boxHeight &&
      position.y + height > y
    );
  }

  // Return the interactions API
  return {
    // Drag state
    isDragging,
    dragStartPosition,
    currentDragPosition,
    draggedElements,
    draggedEdges,
    dragDelta,

    // Selection box state
    selectionBoxActive,
    selectionBoxStart,
    selectionBoxEnd,
    selectionBoxBounds,

    // Edge creation state
    isCreatingEdge,
    edgeStartSocket,
    edgeEndPosition,

    // Pan and zoom state
    x,
    y,
    pan,
    zoomLevel,

    // Cursor and mode
    cursor,
    interactionMode,

    // Events
    lastEvent,

    // Drag methods
    startDrag,
    updateDrag,
    endDrag,
    updateDraggedEdges,

    // Selection box methods
    startSelectionBox,
    updateSelectionBox,
    endSelectionBox,
    isInSelectionBox,

    // Edge creation methods
    startEdgeCreation,
    updateEdgeCreation,
    endEdgeCreation,

    // Pan and zoom methods
    updatePan,
    setZoom,
    zoomIn,
    zoomOut,
    resetZoom,
    resetPan,

    // Mode and cursor methods
    setInteractionMode,
    setCursor,
  };
}

export default useDiagramInteractions;
