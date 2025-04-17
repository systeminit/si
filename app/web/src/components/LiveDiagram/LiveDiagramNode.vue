<template>
  <v-group
    :config="{
      id: `node-${node.id}`,
      x: Number(node.position.x) || 0,
      y: Number(node.position.y) || 0,
      draggable: false,
      name: `node-group-${node.id}`,
    }"
    @mousedown="onMouseDown"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
  >
    <!-- High detail rendering when zoomed in -->
    <template v-if="detailLevel === 'high'">
      <!-- Node background with shadow -->
      <v-rect
        :config="{
          width: Number(node.position.width) || 200,
          height: Number(node.position.height) || 100,
          fill: bgColor,
          stroke: strokeColor,
          strokeWidth: isSelected ? 2 : 1,
          cornerRadius: 4,
          shadowColor: 'rgba(0,0,0,0.3)',
          shadowBlur: 5,
          shadowOffsetX: 2,
          shadowOffsetY: 2,
          shadowOpacity: 0.3,
          name: `node-rect-${node.id}`,
        }"
      />

      <!-- Header bar -->
      <v-rect
        :config="{
          width: Number(node.position.width) || 200,
          height: 28,
          fill: headerColor,
          cornerRadius: [4, 4, 0, 0],
          name: `node-header-${node.id}`,
        }"
      />

      <!-- Title text -->
      <v-text
        :config="{
          x: 10,
          y: 8,
          text: node.title,
          fontSize: 14,
          fontFamily: 'Inter, sans-serif',
          fill: '#FFFFFF',
          width: Number(node.position.width) - 20 || 180,
          ellipsis: true,
          name: `node-title-${node.id}`,
        }"
      />

      <!-- Group/Frame indicator (if applicable) -->
      <template v-if="node.type === 'group'">
        <v-rect
          :config="{
            x: 5,
            y: node.position.height - 22,
            width: node.position.width - 10,
            height: 17,
            fill: 'rgba(255,255,255,0.1)',
            cornerRadius: 3,
          }"
        />
        <v-text
          :config="{
            x: 10,
            y: node.position.height - 20,
            text: `Frame: ${(node as DiagramGroupType).childIds.length} items`,
            fontSize: 11,
            fontFamily: 'Inter, sans-serif',
            fill: '#FFFFFF',
          }"
        />
      </template>
    </template>

    <!-- Medium detail rendering when moderately zoomed out -->
    <template v-else-if="detailLevel === 'medium'">
      <!-- Simplified node background without shadow -->
      <v-rect
        :config="{
          width: Number(node.position.width) || 200,
          height: Number(node.position.height) || 100,
          fill: bgColor,
          stroke: strokeColor,
          strokeWidth: isSelected ? 2 : 1,
          cornerRadius: 4,
          name: `node-rect-${node.id}`,
        }"
      />

      <!-- Header bar -->
      <v-rect
        :config="{
          width: Number(node.position.width) || 200,
          height: 28,
          fill: headerColor,
          cornerRadius: [4, 4, 0, 0],
          name: `node-header-${node.id}`,
        }"
      />

      <!-- Title text (simplified) -->
      <v-text
        :config="{
          x: 10,
          y: 8,
          text: node.title,
          fontSize: 14,
          fontFamily: 'Inter, sans-serif',
          fill: '#FFFFFF',
          width: Number(node.position.width) - 20 || 180,
          ellipsis: true,
          name: `node-title-${node.id}`,
        }"
      />
    </template>

    <!-- Low detail rendering when zoomed far out -->
    <template v-else>
      <!-- Minimal node representation - just a colored rectangle -->
      <v-rect
        :config="{
          width: Number(node.position.width) || 200,
          height: Number(node.position.height) || 100,
          fill: headerColor,
          stroke: isSelected ? SELECTION_COLOR : headerColor,
          strokeWidth: isSelected ? 2 : 0,
          cornerRadius: 2,
          name: `node-rect-${node.id}`,
        }"
      />
    </template>

    <!-- Sockets - only render if node is zoomed in enough or node is selected -->
    <template v-if="isSelected || shouldRenderSockets">
      <LiveDiagramSocketComponent
        v-for="socket in node.sockets"
        :key="socket.id"
        :socket="getSocket(socket.id)"
        :parentX="0"
        :parentY="0"
        :parentWidth="node.position.width"
        :parentHeight="node.position.height"
        :isSelected="isSelected"
        @select="onSocketSelect"
      />
    </template>

    <!-- Selection outline (when dragging) -->
    <v-rect
      v-if="isDragging"
      :config="{
        width: node.position.width,
        height: node.position.height,
        stroke: SELECTION_COLOR,
        strokeWidth: 2,
        dash: [5, 5],
        cornerRadius: 4,
        listening: false,
      }"
    />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, inject, defineAsyncComponent, markRaw } from "vue";
import {
  LiveDiagramNode as DiagramNodeType,
  LiveDiagramGroup as DiagramGroupType,
  LiveDiagramSocket as DiagramSocketType,
  LiveDiagramElementType,
} from "./live_diagram_types";
import { useLiveDiagramContext } from "./LiveDiagram.vue";

// Use async component loading for better performance
const LiveDiagramSocketComponent = defineAsyncComponent(
  () => import("./LiveDiagramSocket.vue"),
);

// Constants
const SELECTION_COLOR = "#4498e0";
const DEFAULT_COLOR = "#2c3e50";

// Props
const props = defineProps<{
  node: DiagramNodeType | DiagramGroupType;
  isSelected?: boolean;
  isHovered?: boolean;
  isDragging?: boolean;
}>();

// Emits
const emit = defineEmits<{
  (e: "select", node: DiagramNodeType | DiagramGroupType): void;
  (e: "hover", node: DiagramNodeType | DiagramGroupType): void;
}>();

// Get the diagram context
const diagramContext = useLiveDiagramContext();
if (!diagramContext) {
  throw new Error(
    "LiveDiagramNode must be used within a LiveDiagram component",
  );
}

const { elements } = diagramContext;

// Compute styles once and cache them by nodeId-selected-hovered state to avoid recalculations
const headerColorCache = new Map<string, string>();
const bgColorCache = new Map<string, string>();
const strokeColorCache = new Map<string, string>();

// Computed styles with caching
const headerColor = computed(() => {
  const cacheKey = props.node.id;
  if (headerColorCache.has(cacheKey)) {
    return headerColorCache.get(cacheKey)!;
  }

  // Use node color with darker shade for the header
  const color = props.node.color || DEFAULT_COLOR;
  headerColorCache.set(cacheKey, color);
  return color;
});

const bgColor = computed(() => {
  const cacheKey = `${props.node.id}-${props.isHovered}`;
  if (bgColorCache.has(cacheKey)) {
    return bgColorCache.get(cacheKey)!;
  }

  let color;
  // For frame/group, use a lighter version of the color with some transparency
  if (props.node.type === LiveDiagramElementType.GROUP) {
    // 20% opacity for the background
    color = props.isHovered
      ? `${props.node.color}30` // 30% opacity when hovered
      : `${props.node.color}20`; // 20% opacity normally
  } else {
    // For regular nodes, use almost white with a slight tint of the node color
    color = props.isHovered
      ? `${props.node.color}10` // 10% tint when hovered
      : `${props.node.color}05`; // 5% tint normally
  }

  bgColorCache.set(cacheKey, color);
  return color;
});

const strokeColor = computed(() => {
  const cacheKey = `${props.node.id}-${props.isSelected}`;
  if (strokeColorCache.has(cacheKey)) {
    return strokeColorCache.get(cacheKey)!;
  }

  const color = props.isSelected ? SELECTION_COLOR : props.node.color;
  strokeColorCache.set(cacheKey, color);
  return color;
});

// Level of detail based on zoom level
// Get current zoom level from diagram context
const { interactions } = diagramContext;
const zoomLevel = computed(() => interactions.zoomLevel.value);

// Constants for level of detail thresholds
const HIGH_DETAIL_THRESHOLD = 0.8; // High detail when zoom >= 0.8
const MEDIUM_DETAIL_THRESHOLD = 0.4; // Medium detail when 0.4 <= zoom < 0.8
const SOCKET_RENDER_ZOOM_THRESHOLD = 0.5; // Only render sockets when zoom >= 0.5

// Determine detail level based on zoom
const detailLevel = computed((): "high" | "medium" | "low" => {
  if (zoomLevel.value >= HIGH_DETAIL_THRESHOLD || props.isSelected) {
    return "high";
  } else if (zoomLevel.value >= MEDIUM_DETAIL_THRESHOLD) {
    return "medium";
  } else {
    return "low";
  }
});

// Only render sockets when zoomed in enough (for performance)
const shouldRenderSockets = computed(() => {
  return zoomLevel.value >= SOCKET_RENDER_ZOOM_THRESHOLD || props.isSelected;
});

// Helper to find a socket by ID
function getSocket(socketId: string): DiagramSocketType {
  const socket = elements.getElementById(socketId) as DiagramSocketType;
  if (!socket) {
    throw new Error(`Socket ${socketId} not found`);
  }
  return socket;
}

// Event handlers
function onMouseDown(_e: MouseEvent) {
  emit("select", props.node);
}

function onMouseOver() {
  emit("hover", props.node);
}

function onMouseOut() {
  // Only needed if we want to do something special on mouse out
}

function onSocketSelect(_socket: DiagramSocketType) {
  // Handle socket selection, possibly starting edge creation
  // TODO: Implement socket selection logic when needed
}
</script>
