<template>
  <div
    class="grow h-full relative bg-neutral-50 dark:bg-neutral-900 overflow-hidden"
    :style="{
      marginLeft: presenceStore.leftResizePanelWidth === 0 
        ? '0' 
        : `${LEFT_PANEL_DRAWER_WIDTH}px`,
      marginRight: presenceStore.rightResizePanelWidth === 0 
        ? '0' 
        : `${RIGHT_PANEL_DRAWER_WIDTH}px`,
    }"
  >
    <!-- Background grid and other elements underneath components -->
    <div class="absolute inset-0 overflow-hidden">
      <v-stage
        v-if="containerWidth > 0 && containerHeight > 0"
        :config="{
          width: containerWidth,
          height: containerHeight,
          scale: { x: zoomLevel, y: zoomLevel },
          offset: { x: gridMinX, y: gridMinY },
          devicePixelRatio: 1,
        }"
      >
        <DiagramGridBackground
          :gridMaxX="gridMaxX"
          :gridMaxY="gridMaxY"
          :gridMinX="gridMinX"
          :gridMinY="gridMinY"
          :zoomLevel="zoomLevel"
        />
      </v-stage>
      
      <!-- Loading state or empty state -->
      <!-- <div
        v-if="fetchDiagramReqStatus.isFirstLoad"
        class="w-full h-full flex items-center bg-[rgba(0,0,0,.1)]"
      >
        <LoadingMessage message="Loading diagram" />
      </div> -->
      <DiagramEmptyState v-else-if="viewsStore.diagramIsEmpty" />
    </div>

    <!-- Main diagram content -->
    <div
      id="konva-container"
      ref="containerRef"
      :style="{ cursor }"
      class="absolute inset-0 overflow-hidden modeling-diagram"
    >
      <!-- Use our custom controls that include the outline and views buttons -->
      <LiveDiagramControls
        :zoomLevel="zoomLevel"
        @zoom-in="interactions.zoomIn"
        @zoom-out="interactions.zoomOut"
        @zoom-reset="resetView"
        @toggle-help="toggleHelpModal"
        @toggle-views="toggleViewsPanel"
        @toggle-assets="toggleAssetsPanel"
        @toggle-outline="toggleOutlinePanel"
        @set-zoom="interactions.setZoom"
        @downloadCanvasScreenshot="downloadCanvasScreenshot" 
        @autoLayout="toggleAutoLayoutMenu"
      />
      
      <!-- Floating panels -->
      <FloatingViewList 
        :isOpen="viewsPanelOpen" 
        @close="viewsPanelOpen = false"
      />
      
      <FloatingDiagramOutline 
        :isOpen="outlinePanelOpen" 
        @close="outlinePanelOpen = false"
        position="top-left" 
        @right-click-item="$emit('right-click-element', $event)"
      />

      <FloatingAssetPanel
        :isOpen="assetPanelOpen"
        @close="assetPanelOpen = false"
        position="top-left"
      />
      
      <!-- We no longer need the simple Auto Layout button as we're using the AutoLayoutExperiment component -->

      <!-- Main stage for diagram rendering -->
      <v-stage
        v-if="containerWidth > 0 && containerHeight > 0"
        ref="stageRef"
        :config="{
          width: containerWidth,
          height: containerHeight,
          scale: { x: zoomLevel, y: zoomLevel },
          offset: { x: gridMinX, y: gridMinY },
          devicePixelRatio: 1,
        }"
        @mousedown="onMouseDown"
        @mousemove="onMouseMove"
        @mouseup="onMouseUp"
        @click="onClick"
        @dblclick="onDoubleClick"
        @click.right="onRightClick"
        @wheel="onWheel"
      >
        
        <!-- Static elements layer (non-moving elements) -->
        <v-layer ref="staticLayer">
          <!-- Nodes and groups -->
          <LiveDiagramNodeComponent
            v-for="node in staticNodes"
            :key="node.id"
            :node="node"
            :isSelected="elements.isElementSelected(node)"
            :isHovered="elements.isElementHovered(node)"
            @select="onNodeSelect"
            @hover="onNodeHover"
          />
          
          <!-- Edges -->
          <LiveDiagramEdgeComponent
            v-for="edge in edges"
            :key="edge.id"
            :edge="edge"
            :isSelected="elements.isElementSelected(edge)"
            :isHovered="elements.isElementHovered(edge)"
            @select="onEdgeSelect"
            @hover="onEdgeHover"
          />
        </v-layer>
        
        <!-- Drag layer (elements being dragged) -->
        <v-layer ref="dragLayer">
          <!-- Dragged nodes -->
          <LiveDiagramNodeComponent
            v-for="node in draggedNodes"
            :key="`drag-${node.id}`"
            :node="node"
            isSelected
            isDragging
          />
          
          <!-- Dragged edges (updated during dragging) -->
          <LiveDiagramEdgeComponent
            v-for="edge in draggedEdges"
            :key="`drag-${edge.id}`"
            :edge="edge"
            isDragging
          />
          
          <!-- Selection box for multi-select -->
          <v-rect
            v-if="
              selectionBoxActive && 
              selectionBoxBounds
            "
            :config="{
              x: selectionBoxBounds.x,
              y: selectionBoxBounds.y,
              width: selectionBoxBounds.width,
              height: selectionBoxBounds.height,
              fill: SELECTION_BOX_INNER_COLOR,
              stroke: SELECTION_COLOR,
              strokeWidth: 1,
              listening: false,
            }"
          />
          
          <!-- Edge being created -->
          <v-line
            v-if="
              isCreatingEdge &&
              edgeStartSocket &&
              edgeEndPosition
            "
            :config="{
              points: [
                getAbsoluteSocketPosition(edgeStartSocket).x,
                getAbsoluteSocketPosition(edgeStartSocket).y,
                edgeEndPosition.x,
                edgeEndPosition.y
              ],
              stroke: EDGE_COLOR,
              strokeWidth: 2,
              lineCap: 'round',
              lineJoin: 'round',
              dash: [5, 5],
              listening: false,
            }"
          />
        </v-layer>
      </v-stage>
      
      <!-- Help modal -->
      <DiagramHelpModal ref="helpModalRef" />
   
    </div>
  </div>
</template>

<script lang="ts">
import { inject, InjectionKey } from 'vue';
import useDiagramElements from "./utils/useDiagramElements";
import useDiagramInteractions from "./utils/useDiagramInteractions";
import useLayoutEngine from "./utils/useLayoutEngine";

// Define the diagram context for child components
interface LiveDiagramContext {
  elements: ReturnType<typeof useDiagramElements>;
  interactions: ReturnType<typeof useDiagramInteractions>;
  layout: ReturnType<typeof useLayoutEngine>;
}
export const LIVE_DIAGRAM_CONTEXT_KEY: InjectionKey<LiveDiagramContext> = Symbol("LiveDiagramContext");
export function useLiveDiagramContext() {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return inject(LIVE_DIAGRAM_CONTEXT_KEY)!;
}
</script>

<script lang="ts" setup>
import { 
  ref, 
  computed, 
  onMounted, 
  onUnmounted,
  ComputedRef,
  provide, 
  watch, 
  nextTick,
Ref
} from "vue";
import { Vector2d } from "konva/lib/types";
import * as _ from "lodash-es";
import { windowListenerManager } from "@si/vue-lib";
import { LoadingMessage } from "@si/vue-lib/design-system";


// Store imports
import { useViewsStore } from "@/store/views.store";
import { useComponentsStore } from "@/store/components.store";
import { usePresenceStore } from "@/store/presence.store";
import { useChangeSetsStore, diagramUlid as clientUlid } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  LiveDiagramSocket,
  LiveDiagramAnyElement,
  LiveDiagramNode as DiagramNodeType,
  LiveDiagramGroup as DiagramGroupType,
  LiveDiagramEdge as DiagramEdgeType,
  LiveDiagramElementType,
  LiveDiagramGroup,
LiveDiagramNode,
} from "./live_diagram_types";

// Component imports
import DiagramGridBackground from "../ModelingDiagram/DiagramGridBackground.vue";
import DiagramEmptyState from "../ModelingDiagram/DiagramEmptyState.vue";
import DiagramHelpModal from "../ModelingDiagram/DiagramHelpModal.vue";
import LiveDiagramNodeComponent from "./LiveDiagramNode.vue";
import LiveDiagramEdgeComponent from "./LiveDiagramEdge.vue";
import LiveDiagramControls from "./LiveDiagramControls.vue";
import FloatingViewList from "./FloatingViewList.vue";
import FloatingDiagramOutline from "./FloatingDiagramOutline.vue";
import FloatingAssetPanel from "./FloatingAssetPanel.vue";

// Import diagram types
import { RightClickElementEvent } from "../ModelingDiagram/diagram_types";


// Constants
const LEFT_PANEL_DRAWER_WIDTH = 320;
const RIGHT_PANEL_DRAWER_WIDTH = 320;
const SELECTION_COLOR = "#4498e0";
const SELECTION_BOX_INNER_COLOR = "rgba(68, 152, 224, 0.1)";
const EDGE_COLOR = "#6b7280";
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 10;
const ZOOM_STEP = 0.1;
const ZOOM_PAN_FACTOR = 1.0; // Factor for panning speed

// No focus mode constants needed

// Component props and emits
const emit = defineEmits<{
  (e: "mouseout"): void;
  (e: "right-click-element", event: RightClickElementEvent): void;
  (e: "close-right-click-menu"): void;
}>();

// Access stores
const viewsStore = useViewsStore();
const componentsStore = useComponentsStore();
const presenceStore = usePresenceStore();
const changeSetsStore = useChangeSetsStore();
const featureFlagsStore = useFeatureFlagsStore();

// Create composables
const elements = useDiagramElements();
const {
  zoomLevel,
  cursor,
  edgeStartSocket,
  edgeEndPosition,
  isCreatingEdge,
  selectionBoxBounds,
  selectionBoxActive,
} = useDiagramInteractions();
const interactions = useDiagramInteractions();
const layout = useLayoutEngine();

// References to DOM and Konva elements
const containerRef = ref<HTMLElement | null>(null);
const stageRef = ref<{ getNode: () => any } | null>(null); // v-stage ref
const staticLayer = ref<{ getNode: () => any } | null>(null); // v-layer ref for static elements
const dragLayer = ref<{ getNode: () => any } | null>(null); // v-layer ref for dragging elements
// No need for these refs anymore
const helpModalRef = ref<{ open: () => void } | null>(null);

// Container dimensions
const containerWidth = ref(0);
const containerHeight = ref(0);

// Floating panel state
const viewsPanelOpen = ref(false);
const outlinePanelOpen = ref(false);
const assetPanelOpen = ref(false);
const autoLayoutMenuOpen = ref(false);

// We manage the canvas origin/panning with gridOrigin
// Following the approach used in ModelingDiagram
const gridOrigin = ref<Vector2d>({ x: 0, y: 0 });

// Save/load gridOrigin from localStorage
watch(
  gridOrigin,
  () => {
    window.localStorage.setItem(
      `si-live-diagram-origin`,
      JSON.stringify(gridOrigin.value),
    );
  },
  { flush: "post" },
);

// Try to load saved grid origin
const savedOrigin = window.localStorage.getItem(`si-live-diagram-origin`);
if (savedOrigin) {
  try {
    const parsed = JSON.parse(savedOrigin);
    if (
      parsed &&
      typeof parsed.x === "number" &&
      typeof parsed.y === "number"
    ) {
      gridOrigin.value = parsed;
    }
  } catch (e) {
    console.warn("Could not parse saved grid origin");
  }
}

// Compute the grid dimensions based on the container size and zoom level
const gridWidth = computed(() => containerWidth.value / zoomLevel.value);
const gridHeight = computed(() => containerHeight.value / zoomLevel.value);

// Compute the visible region bounds based on gridOrigin
const gridMinX = computed(() => gridOrigin.value.x - gridWidth.value / 2);
const gridMaxX = computed(() => gridOrigin.value.x + gridWidth.value / 2);
const gridMinY = computed(() => gridOrigin.value.y - gridHeight.value / 2);
const gridMaxY = computed(() => gridOrigin.value.y + gridHeight.value / 2);

// Loading status
// const fetchDiagramReqStatus = computed(() =>
//   changeSetsStore.getRequestStatus("FETCH_CURRENT_DIAGRAM"),
// );

// Filtered nodes for rendering with optimization
// Using shallowRef and only recomputing when necessary
const nodesList = ref<(LiveDiagramNode | LiveDiagramGroup)[]>([]);

// Update nodes list non-reactively when elements change
function updateNodesList() {
  // Only update the list when not dragging to avoid too many recomputations
  if (!interactions.isDragging.value) {
    nodesList.value = [
      ...Object.values(elements.nodes.value),
      ...Object.values(elements.groups.value),
    ];

    // Ensure all position values are proper numbers (only once when list updates)
    nodesList.value.forEach((node) => {
      node.position.x = Number(node.position.x) || 0;
      node.position.y = Number(node.position.y) || 0;
      node.position.width = Number(node.position.width) || 200;
      node.position.height = Number(node.position.height) || 100;
    });
  }
}

// Call updateNodesList whenever elements change
watch(
  () => [
    Object.keys(elements.nodes.value).length,
    Object.keys(elements.groups.value).length,
  ],
  updateNodesList,
  { immediate: true },
);

// Define a buffer around the visible area to prevent pop-in when panning
const RENDER_BUFFER = 300; // pixels

// Virtualized nodes - only render nodes that are in or near the visible area
const visibleNodes = computed(() => {
  // Calculate the visible area of the grid, with buffer
  const visibleArea = {
    minX: gridMinX.value - RENDER_BUFFER / zoomLevel.value,
    maxX: gridMaxX.value + RENDER_BUFFER / zoomLevel.value,
    minY: gridMinY.value - RENDER_BUFFER / zoomLevel.value,
    maxY: gridMaxY.value + RENDER_BUFFER / zoomLevel.value,
  };

  // Filter nodes that intersect with the visible area
  return nodesList.value.filter((node) => {
    // Skip nodes being dragged
    if (
      interactions.isDragging.value &&
      interactions.draggedElements.value.includes(node.id)
    ) {
      return false;
    }

    // Check if node is in or near the visible area
    const { x, y, width, height } = node.position;
    return (
      x + width >= visibleArea.minX &&
      x <= visibleArea.maxX &&
      y + height >= visibleArea.minY &&
      y <= visibleArea.maxY
    );
  });
});

// Filtered nodes for rendering
const staticNodes = visibleNodes;

const draggedNodes = computed(() => {
  if (!interactions.isDragging.value) return [];

  // Return only the nodes being dragged
  return [
    ...Object.values(elements.nodes.value),
    ...Object.values(elements.groups.value),
  ]
    .filter((node) => interactions.draggedElements.value.includes(node.id))
    .map((node) => {
      // Create a copy with updated position based on drag delta
      const nodeCopy = { ...node };
      nodeCopy.position = {
        ...node.position,
        x: node.position.x + interactions.dragDelta.value.x,
        y: node.position.y + interactions.dragDelta.value.y,
      };
      return nodeCopy;
    });
});

// Edges to show in the drag layer during dragging
const draggedEdges = computed(() => {
  if (!interactions.isDragging.value) return [];

  return interactions.draggedEdges.value
    .map((edge) => {
      // Find the original edge to get properties
      const originalEdge = elements.edges.value[edge.id];
      if (!originalEdge) return null;

      // Return a copy of the edge with updated points
      return {
        ...originalEdge,
        points: edge.points,
      };
    })
    .filter(Boolean);
});

const edges = computed(() => Object.values(elements.edges.value));

// Convert components from store to live diagram elements
function registerStoreElements() {
  // Clear existing elements
  elements.clearElements();

  // Create safe non-reactive copies of data from store
  function safeClone<T>(obj: T): T {
    return JSON.parse(JSON.stringify(obj));
  }

  // Register nodes and groups
  const componentIds = Object.keys(viewsStore.components);
  componentIds.forEach((id) => {
    const component = componentsStore.nodesById[id];
    const rect = viewsStore.components[id];
    if (!component || !rect) return;

    // Create non-reactive copies to avoid circular references
    const safeCopy = safeClone(component);
    const safeRect = safeClone(rect);

    elements.registerComponentFromStore(safeCopy, safeRect);
  });

  const frameIds = Object.keys(viewsStore.groups);
  frameIds.forEach((id) => {
    const component = componentsStore.nodesById[id];
    const pos = viewsStore.groups[id];
    if (!component || !pos) return;

    // Create non-reactive copies to avoid circular references
    const safeCopy = safeClone(component);
    const safePos = safeClone(pos);

    elements.registerComponentFromStore(safeCopy, safePos);
  });

  // Register edges
  Object.values(viewsStore.edges).forEach((edge) => {
    // Create non-reactive copy
    const safeEdge = safeClone(edge);
    elements.registerEdgeFromStore(safeEdge);
  });
}

// Store current layout settings - using a generic Record to avoid tight coupling with Elk internals
const currentLayoutSettings = ref<Record<string, any>>({
  // Default settings
  algorithm: "layered",
  direction: "DOWN",
  spacing: 80,
  "layered.spacing.nodeNodeBetweenLayers": 150,
  "spacing.nodeNode": 80,
  padding: 50,
});

// Function to apply layout settings from AutoLayoutExperiment
function applyLayoutSettings(algorithm: string, settings: any) {
  console.log(`Applying ${algorithm} layout settings:`, settings);
  
  // Create ElkLayoutEngine compatible settings object
  let layoutOptions: Record<string, any> = {};
  
  if (algorithm === "elk") {
    // Map ELK settings directly to layout options
    layoutOptions = {
      algorithm: settings.algorithm || "layered",
      direction: settings.direction || "DOWN",
      spacing: settings.nodeSpacing || 80,
      "layered.spacing.nodeNodeBetweenLayers": settings.layerSpacing || 150,
      "spacing.nodeNode": settings.nodeSpacing || 80,
      padding: settings.padding || 50,
    };
    
    // Add additional settings if they exist
    if (settings.nodePlacement) {
      layoutOptions.nodePlacement = settings.nodePlacement;
    }
    
    if (settings.preventOverlaps !== undefined) {
      layoutOptions["elk.layered.nodePlacement.strategy"] = 
        settings.preventOverlaps ? "INTERACTIVE" : "SIMPLE";
    }
  } else if (algorithm === "dagre") {
    // Convert Dagre settings to layout options
    const direction = settings.rankdir === "TB" ? "DOWN" : 
                      settings.rankdir === "BT" ? "UP" : 
                      settings.rankdir === "LR" ? "RIGHT" : "LEFT";
                      
    layoutOptions = {
      algorithm: "layered",
      direction: direction,
      spacing: settings.nodesep || 80,
      "layered.spacing.nodeNodeBetweenLayers": settings.ranksep || 150,
      "spacing.nodeNode": settings.nodesep || 80,
      padding: settings.marginx || 50,
    };
    
    if (settings.preventOverlaps !== undefined) {
      layoutOptions["elk.layered.nodePlacement.strategy"] = 
        settings.preventOverlaps ? "INTERACTIVE" : "SIMPLE";
    }
  }
  
  // Store the settings for future use
  currentLayoutSettings.value = layoutOptions;
  
  // Apply the layout with the new settings
  computeLayout();
}

// Layout the diagram using ELK or simple grid layout
async function computeLayout(): Promise<void> {
  try {
    // Get all nodes and groups as a single sorted array
    const allNodes = [
      ...Object.values(elements.nodes.value),
      ...Object.values(elements.groups.value),
    ].sort((a, b) => a.id.localeCompare(b.id));

    // Get all edges in sorted order
    const allEdges = Object.values(elements.edges.value).sort((a, b) =>
      a.id.localeCompare(b.id),
    );

    console.log("Available for layout:", {
      nodeCount: allNodes.length,
      edgeCount: allEdges.length,
    });

    // Store initial positions for comparison
    const initialPositions = new Map();
    allNodes.forEach((node) => {
      initialPositions.set(node.id, { ...node.position });
    });

    // Create a graph for the main diagram with current layout settings
    const mainGraphId = "main";
    layout.createGraph(mainGraphId, currentLayoutSettings.value);

    // Add all nodes at once in a single operation
    console.log("Adding nodes to layout engine...");
    layout.addNodes(mainGraphId, allNodes);

    // Add all edges in a single operation
    console.log("Adding edges to layout engine...");
    layout.addEdges(mainGraphId, allEdges);

    // Compute the layout
    console.log("Computing ELK layout...");
    const result = await layout.computeLayout(mainGraphId);
    console.log("ELK Layout computed with:", {
      nodeCount: result.nodes.length,
      edgeCount: result.edges.length,
    });

    // Apply layout results to our elements
    if (result.nodes.length === 0) {
      console.warn("Layout result has no nodes - using a simple grid layout");
      // If we have no layout results, space nodes evenly in a grid
      const nodes = [
        ...Object.values(elements.nodes.value),
        ...Object.values(elements.groups.value),
      ].sort((a, b) => a.id.localeCompare(b.id)); // Keep deterministic order

      // Calculate a grid layout manually
      const columns = Math.ceil(Math.sqrt(nodes.length));
      const spacing = 250; // Space between nodes

      nodes.forEach((node, index) => {
        const col = index % columns;
        const row = Math.floor(index / columns);

        // Update node position to a grid layout
        elements.updateElementPosition(node, {
          x: col * spacing + 50,
          y: row * spacing + 50,
        });
      });
    } else {
      // Apply ELK layout results
      console.log(`Applying layout to ${result.nodes.length} nodes`);

      // First create a backup of existing nodes
      const nodeBackup = {
        nodes: { ...elements.nodes.value },
        groups: { ...elements.groups.value },
      };

      // Clear existing nodes from store
      elements.nodes.value = {};
      elements.groups.value = {};

      // Now recreate each node with the computed layout positions
      result.nodes.forEach((layoutNode) => {
        const originalNode =
          nodeBackup.nodes[layoutNode.id] || nodeBackup.groups[layoutNode.id];
        if (!originalNode) {
          console.warn(
            `Cannot find original node ${layoutNode.id} to recreate`,
          );
          return;
        }

        // Create a fresh copy of the node with new position
        const newNode = {
          ...originalNode,
          position: {
            x: Number(layoutNode.position.x),
            y: Number(layoutNode.position.y),
            width:
              Number(layoutNode.position.width) ||
              Number(originalNode.position.width) ||
              200,
            height:
              Number(layoutNode.position.height) ||
              Number(originalNode.position.height) ||
              100,
          },
        };

        // Register the fresh node back to the appropriate store
        if (newNode.type === LiveDiagramElementType.NODE) {
          elements.nodes.value[newNode.id] = newNode as LiveDiagramNode;
        } else if (newNode.type === LiveDiagramElementType.GROUP) {
          elements.groups.value[newNode.id] = newNode as LiveDiagramGroup;
        }

        // Recreate each socket with the updated position
        layoutNode.sockets.forEach((layoutSocket) => {
          const originalSocket = elements.sockets.value[layoutSocket.id];
          if (!originalSocket) return;

          const newSocket = {
            ...originalSocket,
            position: {
              x: Number(layoutSocket.position.x),
              y: Number(layoutSocket.position.y),
            },
            side: layoutSocket.nodeSide,
          };

          elements.sockets.value[newSocket.id] = newSocket;
        });
      });

      // Update edge points
      result.edges.forEach((layoutEdge) => {
        const edge = elements.getElementById(layoutEdge.id) as
          | DiagramEdgeType
          | undefined;
        if (!edge) return;

        // Convert elk points to vector2d with explicit number conversion
        const points = layoutEdge.points.map((p) => ({
          x: Number(p.x),
          y: Number(p.y),
        }));

        // Create a new edge with updated points to trigger reactivity
        const newEdge = {
          ...edge,
          points,
        };

        // Replace the original edge in the store
        elements.edges.value[edge.id] = newEdge;
      });
    }

    // After layout, update the socket positions for all nodes
    allNodes.forEach((node) => {
      node.sockets.forEach((socket) => {
        if (typeof socket === "string") return;
        elements.updateSocketPosition(socket);
      });
    });

    // After layout, update the view to include all elements
    nextTick(() => {
      fitViewToContent();
    });
  } catch (error) {
    console.error("Error in computeLayout:", error);

    // Fallback to simple grid layout in case of error
    console.warn("Using grid layout as fallback due to error");
    const nodes = [
      ...Object.values(elements.nodes.value),
      ...Object.values(elements.groups.value),
    ];

    // Calculate a grid layout manually
    const columns = Math.ceil(Math.sqrt(nodes.length));
    const spacing = 250; // Space between nodes

    nodes.forEach((node, index) => {
      const col = index % columns;
      const row = Math.floor(index / columns);

      // Update node position to a grid layout
      elements.updateElementPosition(node, {
        x: col * spacing + 50,
        y: row * spacing + 50,
      });
    });

    // Update socket positions
    nodes.forEach((node) => {
      node.sockets.forEach((socket) => {
        if (typeof socket === "string") return;
        elements.updateSocketPosition(socket);
      });
    });

    // Fit view
    nextTick(() => {
      fitViewToContent();
    });
  }
}

// Helper to position the view to show all elements
function fitViewToContent(): void {
  const nodes = [
    ...Object.values(elements.nodes.value),
    ...Object.values(elements.groups.value),
  ];

  if (nodes.length === 0) return;

  // Find bounding box of all nodes
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  nodes.forEach((node) => {
    const { x, y, width, height } = node.position;
    minX = Math.min(minX, x);
    minY = Math.min(minY, y);
    maxX = Math.max(maxX, x + width);
    maxY = Math.max(maxY, y + height);
  });

  // Add padding
  const padding = 50;
  minX -= padding;
  minY -= padding;
  maxX += padding;
  maxY += padding;

  // Calculate center of content
  const contentCenterX = (minX + maxX) / 2;
  const contentCenterY = (minY + maxY) / 2;

  // Calculate content width and height
  const contentWidth = maxX - minX;
  const contentHeight = maxY - minY;

  // Calculate appropriate zoom level to fit content
  const horizontalZoom = containerWidth.value / contentWidth;
  const verticalZoom = containerHeight.value / contentHeight;
  let newZoom = Math.min(horizontalZoom, verticalZoom, 1.0); // Cap at 1.0 zoom
  newZoom = Math.max(newZoom, 0.2); // Ensure minimum zoom of 0.2

  // Set zoom level
  zoomLevel.value = newZoom;
  interactions.setZoom(newZoom);

  // Set gridOrigin to center the content
  gridOrigin.value = {
    x: contentCenterX,
    y: contentCenterY,
  };

  console.log("Fit view - gridOrigin:", gridOrigin.value, "zoom:", newZoom);
}

// Reset view (zoom and pan)
function resetView(): void {
  // Reset zoom level
  zoomLevel.value = 1;
  interactions.setZoom(1);

  // Reset grid origin to center (0, 0)
  gridOrigin.value = { x: 0, y: 0 };

  // Fit content into view
  fitViewToContent();
}

// Helper to get absolute socket position
function getAbsoluteSocketPosition(socket: LiveDiagramSocket): Vector2d {
  const parent = elements.getElementById(socket.parentId) as
    | DiagramNodeType
    | DiagramGroupType
    | undefined;
  if (!parent) return socket.position;

  return {
    x: parent.position.x + socket.position.x,
    y: parent.position.y + socket.position.y,
  };
}

// Get stage point from event - convert from screen to grid coordinates
function getStagePoint(e: { evt?: MouseEvent }): Vector2d | null {
  const stage = stageRef.value?.getNode();
  if (!stage) return null;

  const point = stage.getPointerPosition();
  if (!point) return null;

  // Convert from container coordinates to grid coordinates
  return {
    x: gridMinX.value + point.x / zoomLevel.value,
    y: gridMinY.value + point.y / zoomLevel.value,
  };
}

// Element under point
function getElementAtPoint(point: Vector2d): LiveDiagramAnyElement | null {
  // First check nodes and groups (in reverse order so top elements are first)
  const nodes = [
    ...Object.values(elements.nodes.value),
    ...Object.values(elements.groups.value),
  ].reverse();

  for (const node of nodes) {
    const { x, y, width, height } = node.position;
    if (
      point.x >= x &&
      point.x <= x + width &&
      point.y >= y &&
      point.y <= y + height
    ) {
      // Check if a socket was hit
      for (const socket of node.sockets) {
        const socketAbsPos = getAbsoluteSocketPosition(socket);
        const socketRadius = 5; // Socket hit radius

        const dx = point.x - socketAbsPos.x;
        const dy = point.y - socketAbsPos.y;
        const distance = Math.sqrt(dx * dx + dy * dy);

        if (distance <= socketRadius) {
          return socket;
        }
      }

      return node;
    }
  }

  // Then check edges
  const allEdges = Object.values(elements.edges.value);
  for (const edge of allEdges) {
    // Simple line hit detection
    if (edge.points.length < 2) continue;

    for (let i = 0; i < edge.points.length - 1; i++) {
      const p1 = edge.points[i];
      const p2 = edge.points[i + 1];
      if (!p1 || !p2) continue;
      // Check if point is close to line segment
      const distance = distanceToLineSegment(p1, p2, point);
      if (distance <= 5) {
        // Hit threshold of 5 pixels
        return edge;
      }
    }
  }

  return null;
}

// Calculate distance from point to line segment
function distanceToLineSegment(
  p1: Vector2d,
  p2: Vector2d,
  point: Vector2d,
): number {
  const { x, y } = point;
  const { x: x1, y: y1 } = p1;
  const { x: x2, y: y2 } = p2;

  const A = x - x1;
  const B = y - y1;
  const C = x2 - x1;
  const D = y2 - y1;

  const dot = A * C + B * D;
  const lenSq = C * C + D * D;
  let param = -1;

  if (lenSq !== 0) {
    param = dot / lenSq;
  }

  let xx;
  let yy;

  if (param < 0) {
    xx = x1;
    yy = y1;
  } else if (param > 1) {
    xx = x2;
    yy = y2;
  } else {
    xx = x1 + param * C;
    yy = y1 + param * D;
  }

  const dx = x - xx;
  const dy = y - yy;

  return Math.sqrt(dx * dx + dy * dy);
}

// Event handlers
function onMouseDown(e: { evt?: MouseEvent }) {
  const evt = e.evt as MouseEvent;
  const point = getStagePoint(e);
  if (!point) return;

  // Right button is handled by onRightClick
  if (evt.button === 2) return;

  // Middle button for panning
  if (evt.button === 1) {
    // Start panning
    interactions.startDrag(point, []);
    interactions.setInteractionMode("pan");
    return;
  }

  // Left button for selection and drag
  if (evt.button === 0) {
    
    
    const element = getElementAtPoint(point);

    if (element) {
      // Element clicked

      // Handle socket click (start edge creation)
      if (element.type === LiveDiagramElementType.SOCKET) {
        interactions.startEdgeCreation(element as LiveDiagramSocket, point);
        return;
      }

      // Handle node or group click (selection and drag)
      if (
        element.type === LiveDiagramElementType.NODE ||
        element.type === LiveDiagramElementType.GROUP
      ) {
        // Select the element if not already selected
        if (!elements.isElementSelected(element)) {
          if (evt.shiftKey) {
            // Add to selection
            const newSelection = [
              ...elements.selectedElementIds.value,
              element.id,
            ];
            elements.selectElements(newSelection);
          } else {
            // Replace selection
            elements.selectElement(element.id);
          }
        }

        // Start dragging the selection
        interactions.startDrag(point, elements.selectedElementIds.value);
        return;
      }

      // Handle edge click (selection)
      if (element.type === LiveDiagramElementType.EDGE) {
        if (!evt.shiftKey) {
          elements.selectElement(element.id);
        }
        return;
      }
    } else {
      // Clicked on empty space - start selection box
      if (!evt.shiftKey) {
        elements.clearSelection();
      }
      interactions.startSelectionBox(point);
    }
  }
}

function onMouseMove(e: { evt?: MouseEvent }) {
  const point = getStagePoint(e);
  if (!point) return;

  // Handle component insertion cursor when dragging
  if (componentsStore.selectedInsertCategoryVariantId) {
    cursor.value = "crosshair"; // Use crosshair cursor to indicate insertion
  }

  // Update hover state
  const hoveredElement = getElementAtPoint(point);
  elements.setHoveredElement(hoveredElement?.id || null);

  // Handle drag update
  if (interactions.isDragging.value) {
    if (interactions.interactionMode.value === "pan") {
      // Handle panning with gridOrigin approach
      const stage = stageRef.value?.getNode();
      if (!stage) return;

      // Get the raw pointer position from the stage
      const pointer = stage.getPointerPosition();
      if (!pointer) return;

      // Get the last pointer position
      const lastPointer = interactions.currentDragPosition.value;
      if (!lastPointer) {
        // Initialize with current position
        interactions.currentDragPosition.value = { x: pointer.x, y: pointer.y };
        return;
      }

      // Calculate the delta in screen coordinates
      const deltaX = pointer.x - lastPointer.x;
      const deltaY = pointer.y - lastPointer.y;

      // Apply delta to gridOrigin, accounting for zoom level
      const panFactor = 1 / zoomLevel.value;
      gridOrigin.value = {
        x: gridOrigin.value.x - deltaX * panFactor,
        y: gridOrigin.value.y - deltaY * panFactor,
      };

      // Store current raw pointer position for next move
      interactions.currentDragPosition.value = { x: pointer.x, y: pointer.y };
    } else {
      // Handle dragging elements
      interactions.updateDrag(point);

      // Update dragged edges for each dragged node
      if (interactions.draggedElements.value.length > 0) {
        interactions.draggedElements.value.forEach((nodeId) => {
          interactions.updateDraggedEdges(
            nodeId,
            elements.nodes.value,
            elements.groups.value,
            elements.sockets.value,
            elements.edges.value,
            interactions.dragDelta.value,
          );
        });
      }
    }
    return;
  }

  // Handle selection box update
  if (interactions.selectionBoxActive.value) {
    interactions.updateSelectionBox(point);
    return;
  }

  // Handle edge creation update
  if (interactions.isCreatingEdge.value) {
    interactions.updateEdgeCreation(point);
    return;
  }
}

function onMouseUp(e: { evt?: MouseEvent }) {
  const point = getStagePoint(e);
  if (!point) return;

  // Handle component insertion on drop
  if (componentsStore.selectedInsertCategoryVariantId) {
    const selectedVariant = componentsStore.categoryVariantById[
      componentsStore.selectedInsertCategoryVariantId
    ];
    
    if (selectedVariant) {
      // Create a new component at position 0,0
      const fixedPoint = { x: 0, y: 0 };
      createComponent(selectedVariant, fixedPoint);
      componentsStore.cancelInsert();
      return;
    }
  }

  // Handle drag end
  if (interactions.isDragging.value) {
    if (interactions.interactionMode.value === "pan") {
      // End panning
      interactions.endDrag();
      interactions.setInteractionMode("select");
    } else if (interactions.draggedElements.value.length > 0) {
      // Apply drag changes to elements
      const delta = interactions.dragDelta.value;

      // Update positions of dragged elements
      interactions.draggedElements.value.forEach((id) => {
        const element = elements.getElementById(id) as
          | DiagramNodeType
          | DiagramGroupType
          | undefined;
        if (!element) return;

        elements.updateElementPosition(element, {
          x: element.position.x + delta.x,
          y: element.position.y + delta.y,
        });

        // Update any edges connected to this node
        elements.updateConnectedEdges(id);
      });

      // End drag operation
      interactions.endDrag();
    }
    return;
  }

  // Handle selection box end
  if (interactions.selectionBoxActive.value) {
    const bounds = interactions.endSelectionBox();
    if (!bounds) return;

    // Find elements inside the selection box
    const selectedIds = [
      ...Object.values(elements.nodes.value),
      ...Object.values(elements.groups.value),
    ]
      .filter(
        (node) =>
          bounds.x <= node.position.x + node.position.width &&
          bounds.x + bounds.width >= node.position.x &&
          bounds.y <= node.position.y + node.position.height &&
          bounds.y + bounds.height >= node.position.y,
      )
      .map((node) => node.id);

    // Update selection
    if (e.evt?.shiftKey) {
      // Add to existing selection
      const newSelection = [
        ...elements.selectedElementIds.value,
        ...selectedIds.filter(
          (id) => !elements.selectedElementIds.value.includes(id),
        ),
      ];
      elements.selectElements(newSelection);
    } else {
      // Replace selection
      elements.selectElements(selectedIds);
    }
    return;
  }

  // Handle edge creation end
  if (interactions.isCreatingEdge.value) {
    const targetElement = getElementAtPoint(point);

    if (
      targetElement &&
      targetElement.type === LiveDiagramElementType.SOCKET &&
      interactions.edgeStartSocket.value?.id !== targetElement.id
    ) {
      // Valid connection between two sockets
      interactions.endEdgeCreation(targetElement as LiveDiagramSocket);

      // TODO: Create actual edge in the store
    } else {
      // Invalid target, cancel edge creation
      interactions.endEdgeCreation();
    }
    return;
  }
}

function onClick(e: { evt?: MouseEvent }) {
  const point = getStagePoint(e);
  if (!point) return;

  const clickedElement = getElementAtPoint(point);

  // If element is null, clear selection unless shift key is pressed
  if (!clickedElement && !e.evt?.shiftKey) {
    elements.clearSelection();
  }
}

function onDoubleClick(e: { evt?: MouseEvent }) {
  const point = getStagePoint(e);
  if (!point) return;

  const element = getElementAtPoint(point);

  // Double click on group/frame to zoom to its content
  if (element?.type === LiveDiagramElementType.GROUP) {
    const group = element as LiveDiagramGroup;

    // Center view on the group
    const { x, y, width, height } = group.position;
    const stage = stageRef.value?.getNode();
    if (!stage) return;

    // Calculate zoom level to fit the group
    const padding = 50;
    const scaleX = (containerWidth.value - padding * 2) / width;
    const scaleY = (containerHeight.value - padding * 2) / height;
    const newZoom = Math.min(scaleX, scaleY, MAX_ZOOM);

    // Set new zoom and center on group
    interactions.setZoom(newZoom);
    interactions.updatePan(
      x + width / 2 - containerWidth.value / (2 * newZoom),
      y + height / 2 - containerHeight.value / (2 * newZoom),
    );
  }
}

function onRightClick(e: { evt?: MouseEvent }) {
  const evt = e.evt as MouseEvent;
  const point = getStagePoint(e);
  if (!point) return;

  const element = getElementAtPoint(point);

  if (element) {
    // Map to original component data format for backward compatibility
    let originalData = null;

    if (
      element.type === LiveDiagramElementType.NODE ||
      element.type === LiveDiagramElementType.GROUP
    ) {
      const componentId = element.id;
      originalData = componentsStore.allComponentsById[componentId];
    } else if (element.type === LiveDiagramElementType.EDGE) {
      const edgeId = element.id;
      originalData = componentsStore.diagramEdgesById[edgeId];
    }

    if (originalData) {
      emit("right-click-element", {
        element: originalData,
        e: evt,
      });
    }
  }
}

function onWheel(e: { evt?: WheelEvent }) {
  const evt = e.evt as WheelEvent;
  evt.preventDefault();

  // Close right-click menu if open
  emit("close-right-click-menu");

  // Is it a mouse wheel or a trackpad pinch to zoom?
  const isTrackpadPinch = !_.isInteger(evt.deltaY);

  // If META key (CMD) or CTRL key with trackpad pinch, treat as zoom; otherwise pan
  if (evt.metaKey || (evt.ctrlKey && isTrackpadPinch)) {
    // Zoom functionality
    const stage = stageRef.value?.getNode();
    if (!stage) return;

    // Get pointer position
    const pointer = stage.getPointerPosition();
    if (!pointer) return;

    // Adjust zoom sensitivity based on whether it's trackpad pinch
    const zoomSpeed = 0.001 * (isTrackpadPinch ? 20 : 1) * zoomLevel.value;

    // Calculate new zoom level
    let newZoom = zoomLevel.value - evt.deltaY * zoomSpeed;
    if (newZoom < MIN_ZOOM) newZoom = MIN_ZOOM;
    if (newZoom > MAX_ZOOM) newZoom = MAX_ZOOM;

    // Calculate grid coordinates of pointer before zoom change
    // This is the world coordinate that we want to keep fixed
    const beforeZoomGridX = gridMinX.value + pointer.x / zoomLevel.value;
    const beforeZoomGridY = gridMinY.value + pointer.y / zoomLevel.value;

    // Calculate what the grid coordinates would be after zoom change
    // if we didn't adjust the origin
    const newGridWidth = containerWidth.value / newZoom;
    const newGridMinX = gridOrigin.value.x - newGridWidth / 2;
    const newGridHeight = containerHeight.value / newZoom;
    const newGridMinY = gridOrigin.value.y - newGridHeight / 2;

    // Calculate what pointer's grid coords would be after zoom
    const afterZoomGridX = newGridMinX + pointer.x / newZoom;
    const afterZoomGridY = newGridMinY + pointer.y / newZoom;

    // Adjust grid origin to keep the pointer position fixed in world coordinates
    gridOrigin.value = {
      x: gridOrigin.value.x - (afterZoomGridX - beforeZoomGridX),
      y: gridOrigin.value.y - (afterZoomGridY - beforeZoomGridY),
    };

    // Set the new zoom level
    zoomLevel.value = newZoom;
    interactions.setZoom(newZoom);

    // Store zoom level for persistence
    if (newZoom === 1) {
      window.localStorage.removeItem("si-diagram-zoom");
    } else {
      window.localStorage.setItem("si-diagram-zoom", `${newZoom}`);
    }
  } else {
    // Pan functionality - use the wheel to pan diagram
    const panFactor = ZOOM_PAN_FACTOR / zoomLevel.value;

    // Update grid origin based on wheel deltas
    gridOrigin.value = {
      x: gridOrigin.value.x + evt.deltaX * panFactor,
      y: gridOrigin.value.y + evt.deltaY * panFactor,
    };
  }
}

// Selection handlers
function onNodeSelect(node: DiagramNodeType | DiagramGroupType) {
  elements.selectElement(node.id);

  // Update the view store for backward compatibility
  if (node.originalData) {
    viewsStore.setSelectedComponentId(node.id);
  }
}

function onEdgeSelect(edge: DiagramEdgeType) {
  elements.selectElement(edge.id);

  // Update the view store for backward compatibility
  if (edge.originalData) {
    viewsStore.setSelectedEdgeId(edge.id);
  }
}

function onNodeHover(node: DiagramNodeType | DiagramGroupType) {
  elements.setHoveredElement(node.id);
}

function onEdgeHover(edge: DiagramEdgeType) {
  elements.setHoveredElement(edge.id);
}



// Screenshot function
function downloadCanvasScreenshot(): void {
  const stage = stageRef.value?.getNode();
  if (!stage) return;

  const dataURL = stage.toDataURL();
  const a = document.createElement("a");
  a.href = dataURL;
  a.download = `diagram-${new Date().toISOString()}.png`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
}

// Help modal
function toggleHelpModal(): void {
  helpModalRef.value?.open();
}

// Toggle views panel
function toggleViewsPanel(): void {
  viewsPanelOpen.value = !viewsPanelOpen.value;
  // Close the other panel when opening this one
  if (viewsPanelOpen.value) {
    outlinePanelOpen.value = false;
    autoLayoutMenuOpen.value = false;
  }
}
function toggleAssetsPanel(): void {
  assetPanelOpen.value = !assetPanelOpen.value;
  // Close the other panels when opening this one
  if (assetPanelOpen.value) {
    outlinePanelOpen.value = false;
    viewsPanelOpen.value = false;
    autoLayoutMenuOpen.value = false;
  }
}

// Toggle auto layout menu panel
function toggleAutoLayoutMenu(): void {
  autoLayoutMenuOpen.value = !autoLayoutMenuOpen.value;
  // Close other panels when opening this one
  if (autoLayoutMenuOpen.value) {
    outlinePanelOpen.value = false;
    viewsPanelOpen.value = false;
    assetPanelOpen.value = false;
  }
}

// Toggle outline panel
function toggleOutlinePanel(): void {
  outlinePanelOpen.value = !outlinePanelOpen.value;
  // Close the other panel when opening this one
  if (outlinePanelOpen.value) {
    viewsPanelOpen.value = false;
    autoLayoutMenuOpen.value = false;
  }
}

// Provide the diagram context to child components
provide(LIVE_DIAGRAM_CONTEXT_KEY, {
  elements,
  interactions,
  layout,
});

// Lifecycle hooks
onMounted(() => {
  // Update container dimensions
  const updateSize = () => {
    if (containerRef.value) {
      containerWidth.value = containerRef.value.offsetWidth;
      containerHeight.value = containerRef.value.offsetHeight;
    }
  };

  updateSize();

  // Add resize observer
  const resizeObserver = new ResizeObserver(updateSize);
  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }

  // Initialize zoom level from localStorage if available
  const storedZoom = window.localStorage.getItem("si-diagram-zoom");
  if (storedZoom) {
    try {
      const zoomValue = parseFloat(storedZoom);
      if (!isNaN(zoomValue) && zoomValue >= MIN_ZOOM && zoomValue <= MAX_ZOOM) {
        interactions.setZoom(zoomValue);
      }
    } catch (e) {
      console.warn("Failed to parse stored zoom level:", e);
    }
  }

  // Register components from the store
  registerStoreElements();

  // Compute initial layout
  computeLayout();

  // Setup watchers
  watch(
    [() => componentsStore.allComponentsById, () => viewsStore.edges],
    () => {
      // Register new components when store changes
      registerStoreElements();
      computeLayout();
    },
    { deep: true },
  );

  // Sync view store selection with our selection
  watch(elements.selectedElementIds, (newSelectedIds) => {
    // When selection changes, notify view store
    if (newSelectedIds.length === 1) {
      const id = newSelectedIds[0] ?? "";
      const element = elements.getElementById(id);
      if (
        element?.type === LiveDiagramElementType.NODE ||
        element?.type === LiveDiagramElementType.GROUP
      ) {
        viewsStore.setSelectedComponentId(id);
      } else if (element?.type === LiveDiagramElementType.EDGE) {
        viewsStore.setSelectedEdgeId(id);
      }
    } else if (newSelectedIds.length > 1) {
      // Multi-select
      viewsStore.selectedComponentIds = newSelectedIds;
    } else {
      // Clear selection
      viewsStore.selectedComponentIds = [];
      viewsStore.selectedEdgeId = null;
    }
  });

  // Add keyboard listeners
  const handleKeyDown = (e: KeyboardEvent) => {
    // Escape key to cancel operations
    if (e.key === "Escape") {
      if (interactions.isCreatingEdge.value) {
        interactions.endEdgeCreation();
      }

      if (interactions.selectionBoxActive.value) {
        interactions.endSelectionBox();
      }

      if (interactions.isDragging.value) {
        interactions.endDrag();
      }
    }

    // Delete key to remove selected elements
    if (e.key === "Delete" || e.key === "Backspace") {
      if (elements.selectedElementIds.value.length > 0) {
        // TODO: Implement deletion of selected elements
      }
    }
  };

  windowListenerManager.addEventListener("keydown", handleKeyDown);

  onUnmounted(() => {
    resizeObserver.disconnect();
    windowListenerManager.removeEventListener("keydown", handleKeyDown);
  });
});
</script>

<style scoped>
.live-diagram {
  position: relative;
  touch-action: none;
}
</style>
