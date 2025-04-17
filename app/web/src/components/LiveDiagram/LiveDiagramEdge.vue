<template>
  <v-group
    :config="{
      id: `edge-${edge.id}`,
      opacity: isDragging ? 0.7 : 1,
    }"
    @mousedown="!isDragging && onMouseDown"
    @mouseover="!isDragging && onMouseOver"
    @mouseout="!isDragging && onMouseOut"
  >
    <!-- Base edge line -->
    <v-line
      :config="{
        points: flattenedPoints,
        stroke: lineColor,
        strokeWidth: isSelected || isHovered ? 2 : 1,
        lineCap: 'round',
        lineJoin: 'round',
        shadowEnabled: isSelected,
        shadowColor: SELECTION_COLOR,
        shadowBlur: 4,
        shadowOpacity: 0.5,
        dash: edge.isManagement ? [6, 2] : undefined,
      }"
    />

    <!-- Arrow head for directional edges -->
    <v-group
      v-if="!edge.isBidirectional"
      :config="{
        x: arrowHeadPosition.x,
        y: arrowHeadPosition.y,
        rotation: arrowHeadRotation,
      }"
    >
      <v-line
        :config="{
          points: [-6, -4, 0, 0, -6, 4],
          stroke: lineColor,
          strokeWidth: isSelected || isHovered ? 2 : 1,
          lineCap: 'round',
          lineJoin: 'round',
        }"
      />
    </v-group>

    <!-- Bidirectional indicator (dots on both ends) -->
    <template v-if="edge.isBidirectional">
      <!-- Start dot -->
      <v-circle
        :config="{
          x: edge.points[0]?.x,
          y: edge.points[0]?.y,
          radius: 3,
          fill: lineColor,
          listening: false,
        }"
      />

      <!-- End dot -->
      <v-circle
        :config="{
          x: edge.points[edge.points.length - 1]?.x,
          y: edge.points[edge.points.length - 1]?.y,
          radius: 3,
          fill: lineColor,
          listening: false,
        }"
      />
    </template>

    <!-- Selection highlight when edge is selected -->
    <v-line
      v-if="isSelected"
      :config="{
        points: flattenedPoints,
        stroke: SELECTION_COLOR,
        strokeWidth: 4,
        lineCap: 'round',
        lineJoin: 'round',
        opacity: 0.3,
        listening: false,
      }"
    />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, inject } from "vue";
import { Vector2d } from "konva/lib/types";
import { LiveDiagramEdge } from "./live_diagram_types";
import { LIVE_DIAGRAM_CONTEXT_KEY } from "./LiveDiagram.vue";

// Constants
// Default colors for different edge states
const MANAGEMENT_COLOR = "#A78BFA"; // Color for management sockets
const SELECTION_COLOR = "#4498e0"; // Blue selection color
const HOVER_COLOR = "#60A5FA"; // Light blue hover color
const EDGE_COLOR = "#6b7280"; // Default edge color

// Props
const props = defineProps<{
  edge: LiveDiagramEdge;
  isSelected?: boolean;
  isHovered?: boolean;
  isDragging?: boolean;
}>();

// Emits
const emit = defineEmits<{
  (e: "select", edge: LiveDiagramEdge): void;
  (e: "hover", edge: LiveDiagramEdge): void;
}>();

// Get diagram context
const diagramContext = inject(LIVE_DIAGRAM_CONTEXT_KEY);
if (!diagramContext) {
  throw new Error(
    "LiveDiagramEdge must be used within a LiveDiagram component",
  );
}

// Access diagram context elements if needed

// Flatten points for the line component (convert Vector2d[] to number[])
const flattenedPoints = computed<number[]>(() => {
  return props.edge.points.flatMap((point) => [point.x, point.y]);
});

// Edge color based on edge type and state
const lineColor = computed<string>(() => {
  if (props.isSelected) {
    return SELECTION_COLOR;
  }
  if (props.isHovered) {
    return HOVER_COLOR;
  }
  if (props.isDragging) {
    // Use a lighter color for edges being dragged
    return "#90cdf4"; // Light blue
  }
  if (props.edge.isManagement) {
    return MANAGEMENT_COLOR;
  }
  return EDGE_COLOR;
});

// Calculate the position and rotation for the arrow head
const arrowHeadPosition = computed<Vector2d>(() => {
  // Get the end point of the edge
  const points = props.edge.points;
  return points[points.length - 1];
});

const arrowHeadRotation = computed<number>(() => {
  // Calculate the angle between the last two points
  const points = props.edge.points;
  if (points.length < 2) return 0;

  const lastPoint = points[points.length - 1] ?? null;
  const secondLastPoint = points[points.length - 2] ?? null;

  const dx = lastPoint.x - secondLastPoint.x;
  const dy = lastPoint.y - secondLastPoint.y;

  // Calculate angle in degrees
  return (Math.atan2(dy, dx) * 180) / Math.PI;
});

// Event handlers
function onMouseDown(e: { cancelBubble?: boolean }) {
  emit("select", props.edge);
  e.cancelBubble = true; // Stop event from bubbling
}

function onMouseOver() {
  emit("hover", props.edge);
}

function onMouseOut() {
  // Handle mouse out if needed
}
</script>
