<template>
  <v-group
    :config="{
      id: `socket-${socket.id}`,
      x: socketPosition.x,
      y: socketPosition.y,
    }"
    @mousedown="onMouseDown"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
  >
    <!-- Socket connector circle -->
    <v-circle
      :config="{
        radius: isHovered || isSelected ? 6 : 5,
        fill: socket.isManagement ? MANAGEMENT_COLOR : SOCKET_COLOR,
        stroke: isHovered
          ? HOVER_COLOR
          : isSelected
          ? SELECTION_COLOR
          : 'rgba(0,0,0,0.2)',
        strokeWidth: isHovered || isSelected ? 2 : 1,
      }"
    />

    <!-- Socket label (only shown on hover or if explicitly configured) -->
    <v-text
      v-if="(isHovered || isSelected || alwaysShowLabel) && socket.label"
      :config="{
        x: labelOffset.x,
        y: labelOffset.y,
        text: socket.label,
        fontSize: 11,
        fontFamily: 'Inter, sans-serif',
        fill: '#FFFFFF',
        align: labelAlign,
        listening: false,
      }"
    />

    <!-- Required indicator -->
    <v-circle
      v-if="socket.isRequired"
      :config="{
        x: requiredDotPosition.x,
        y: requiredDotPosition.y,
        radius: 2,
        fill: '#FF5555',
        listening: false,
      }"
    />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, inject } from "vue";
import { Vector2d } from "konva/lib/types";
import { LiveDiagramSocket } from "./live_diagram_types";
import { useLiveDiagramContext } from "./LiveDiagram.vue";

// Constants
const SOCKET_COLOR = "#9BA3AF"; // Default socket color
const MANAGEMENT_COLOR = "#A78BFA"; // Color for management sockets
const SELECTION_COLOR = "#4498e0"; // Blue selection color
const HOVER_COLOR = "#60A5FA"; // Light blue hover color

// Props
const props = defineProps<{
  socket: LiveDiagramSocket;
  parentX: number;
  parentY: number;
  parentWidth: number;
  parentHeight: number;
  isSelected?: boolean;
  isHovered?: boolean;
  alwaysShowLabel?: boolean;
}>();

// Emits
const emit = defineEmits<{
  (e: "select", socket: LiveDiagramSocket): void;
  (e: "hover", socket: LiveDiagramSocket): void;
}>();

// Get diagram context
const diagramContext = useLiveDiagramContext();
if (!diagramContext) {
  throw new Error(
    "LiveDiagramSocket must be used within a LiveDiagram component",
  );
}

// Calculate socket position based on the socket's side and its parent node
const socketPosition = computed<Vector2d>(() => {
  const { side, position } = props.socket;
  // If the socket already has a position, use it
  if (position && position.x !== undefined && position.y !== undefined) {
    return position;
  }

  // Otherwise calculate position based on side
  switch (side) {
    case "left":
      return { x: 0, y: props.parentHeight / 2 };
    case "right":
      return { x: props.parentWidth, y: props.parentHeight / 2 };
    case "top":
      return { x: props.parentWidth / 2, y: 0 };
    case "bottom":
      return { x: props.parentWidth / 2, y: props.parentHeight };
    default:
      return { x: 0, y: 0 };
  }
});

// Calculate label position
const labelOffset = computed<Vector2d>(() => {
  const { side } = props.socket;
  const offset = 8; // Distance from socket to label

  switch (side) {
    case "left":
      return { x: offset, y: -8 };
    case "right":
      return { x: -offset, y: -8 };
    case "top":
      return { x: 0, y: offset };
    case "bottom":
      return { x: 0, y: -offset - 10 };
    default:
      return { x: 0, y: 0 };
  }
});

// Text alignment based on socket side
const labelAlign = computed(() => {
  const { side } = props.socket;

  switch (side) {
    case "left":
      return "left";
    case "right":
      return "right";
    default:
      return "center";
  }
});

// Position for the required indicator dot
const requiredDotPosition = computed<Vector2d>(() => {
  const { side } = props.socket;
  const offset = 3; // Distance from socket to required indicator

  switch (side) {
    case "left":
      return { x: offset, y: -offset };
    case "right":
      return { x: -offset, y: -offset };
    case "top":
      return { x: offset, y: offset };
    case "bottom":
      return { x: offset, y: -offset };
    default:
      return { x: 0, y: 0 };
  }
});

// Event handlers
function onMouseDown(e: { cancelBubble?: boolean }) {
  emit("select", props.socket);
  e.cancelBubble = true; // Stop event from bubbling up to parent node
}

function onMouseOver() {
  emit("hover", props.socket);
}

function onMouseOut() {
  // Handle mouse out if needed
}
</script>
