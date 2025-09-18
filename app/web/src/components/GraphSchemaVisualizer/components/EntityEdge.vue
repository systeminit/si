<template>
  <g :class="edgeClasses">
    <!-- Main edge path -->
    <path
      :d="pathData"
      :class="pathClasses"
      :marker-end="markerUrl"
      fill="none"
    />
    
    <!-- Edge label (relationship type) -->
    <g v-if="showLabel && labelPosition" :transform="`translate(${labelPosition.x}, ${labelPosition.y})`">
      <!-- Label background -->
      <rect
        :x="-labelWidth / 2"
        :y="-8"
        :width="labelWidth"
        height="16"
        :class="labelBgClasses"
        rx="8"
      />
      
      <!-- Label text -->
      <text
        text-anchor="middle"
        y="3"
        :class="labelTextClasses"
        font-size="10"
      >
        {{ relationshipLabel }}
      </text>
    </g>
    
    <!-- Hover interaction area (wider than visible path) -->
    <path
      :d="pathData"
      class="edge-interaction"
      stroke="transparent"
      stroke-width="10"
      fill="none"
      @mouseenter="handleMouseEnter"
      @mouseleave="handleMouseLeave"
      @click="handleClick"
    />
  </g>
</template>

<script setup lang="ts">
import { computed } from "vue";
import * as d3 from "d3";
import type { RelationshipEdge, EntityNode } from "../types/schema-graph";

interface Props {
  edge: RelationshipEdge;
  sourceNode?: EntityNode;
  targetNode?: EntityNode;
  isHighlighted?: boolean;
  isSelected?: boolean;
  showLabel?: boolean;
}

interface Emits {
  (e: "click", edge: RelationshipEdge, event: MouseEvent): void;
  (e: "mouseenter", edge: RelationshipEdge): void;
  (e: "mouseleave", edge: RelationshipEdge): void;
}

const props = withDefaults(defineProps<Props>(), {
  isHighlighted: false,
  isSelected: false,
  showLabel: true
});

const emit = defineEmits<Emits>();

// Calculate connection points
const sourcePoint = computed(() => {
  if (!props.sourceNode || !props.targetNode) {
    return { x: 0, y: 0 };
  }
  
  const sourceCenter = {
    x: props.sourceNode.position.x + props.sourceNode.dimensions.width / 2,
    y: props.sourceNode.position.y + props.sourceNode.dimensions.height / 2
  };
  const targetCenter = {
    x: props.targetNode.position.x + props.targetNode.dimensions.width / 2,
    y: props.targetNode.position.y + props.targetNode.dimensions.height / 2
  };
  
  // Calculate exit point on source node edge
  return getNodeConnectionPoint(props.sourceNode, sourceCenter, targetCenter);
});

const targetPoint = computed(() => {
  if (!props.sourceNode || !props.targetNode) {
    return { x: 0, y: 0 };
  }
  
  const sourceCenter = {
    x: props.sourceNode.position.x + props.sourceNode.dimensions.width / 2,
    y: props.sourceNode.position.y + props.sourceNode.dimensions.height / 2
  };
  const targetCenter = {
    x: props.targetNode.position.x + props.targetNode.dimensions.width / 2,
    y: props.targetNode.position.y + props.targetNode.dimensions.height / 2
  };
  
  // Calculate entry point on target node edge
  return getNodeConnectionPoint(props.targetNode, targetCenter, sourceCenter);
});

// Generate path data for the edge
const pathData = computed(() => {
  const source = sourcePoint.value;
  const target = targetPoint.value;
  
  // Create a smooth curved path
  const dx = target.x - source.x;
  const dy = target.y - source.y;
  
  // Control points for bezier curve
  const controlOffset = Math.min(Math.abs(dx), Math.abs(dy)) * 0.5;
  const cp1x = source.x + (dx > 0 ? controlOffset : -controlOffset);
  const cp1y = source.y;
  const cp2x = target.x - (dx > 0 ? controlOffset : -controlOffset);
  const cp2y = target.y;
  
  return `M ${source.x} ${source.y} C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${target.x} ${target.y}`;
});

// Calculate label position (midpoint of path)
const labelPosition = computed(() => {
  if (!props.sourceNode || !props.targetNode) {
    return { x: 0, y: 0 };
  }
  
  const source = sourcePoint.value;
  const target = targetPoint.value;
  
  return {
    x: (source.x + target.x) / 2,
    y: (source.y + target.y) / 2
  };
});

// Relationship type display
const relationshipLabel = computed(() => {
  // Use arity to determine the relationship display
  switch (props.edge.arity) {
    case 'one': return '1:1';
    case 'many': return '1:N';
    default: return 'REL';
  }
});

const labelWidth = computed(() => {
  return relationshipLabel.value.length * 6 + 12; // Approximate width
});

// Theme detection
const isDarkTheme = computed(() => {
  return document.documentElement.classList.contains('dark');
});

// Marker URL for arrow
const markerUrl = computed(() => {
  return `url(#arrow-${isDarkTheme.value ? 'dark' : 'light'}-${props.isHighlighted ? 'highlight' : 'normal'})`;
});

// Styling classes
const edgeClasses = computed(() => [
  'entity-edge',
  'transition-all duration-200',
  props.isSelected && 'selected'
]);

const pathClasses = computed(() => {
  const baseClasses = ['transition-all duration-200'];
  
  if (props.isHighlighted) {
    return baseClasses.concat([
      isDarkTheme.value ? 'stroke-blue-400' : 'stroke-blue-500',
      'stroke-2'
    ]);
  }
  
  return baseClasses.concat([
    isDarkTheme.value ? 'stroke-neutral-500' : 'stroke-neutral-400',
    'stroke-1',
    'opacity-70'
  ]);
});

const labelBgClasses = computed(() => [
  'transition-all duration-200',
  isDarkTheme.value ? 'fill-neutral-800' : 'fill-white',
  isDarkTheme.value ? 'stroke-neutral-600' : 'stroke-neutral-300',
  'stroke-1'
]);

const labelTextClasses = computed(() => [
  'font-mono font-semibold',
  isDarkTheme.value ? 'fill-neutral-200' : 'fill-neutral-700'
]);

// Helper function to find connection point on node edge
function getNodeConnectionPoint(
  node: EntityNode, 
  nodeCenter: {x: number, y: number}, 
  otherCenter: {x: number, y: number}
) {
  const dx = otherCenter.x - nodeCenter.x;
  const dy = otherCenter.y - nodeCenter.y;
  
  const nodeLeft = node.position.x;
  const nodeRight = node.position.x + node.dimensions.width;
  const nodeTop = node.position.y;
  const nodeBottom = node.position.y + node.dimensions.height;
  
  // Determine which edge of the rectangle to connect to
  const absRatioX = Math.abs(dx / node.dimensions.width);
  const absRatioY = Math.abs(dy / node.dimensions.height);
  
  if (absRatioX > absRatioY) {
    // Connect to left or right edge
    const x = dx > 0 ? nodeRight : nodeLeft;
    const y = nodeCenter.y;
    return { x, y };
  } else {
    // Connect to top or bottom edge
    const x = nodeCenter.x;
    const y = dy > 0 ? nodeBottom : nodeTop;
    return { x, y };
  }
}

// Event handlers
const handleClick = (event: MouseEvent) => {
  emit("click", props.edge, event);
};

const handleMouseEnter = () => {
  emit("mouseenter", props.edge);
};

const handleMouseLeave = () => {
  emit("mouseleave", props.edge);
};
</script>

<style scoped>
.entity-edge {
  cursor: pointer;
}

.edge-interaction {
  cursor: pointer;
}

.entity-edge.selected path {
  stroke-dasharray: 4 2;
  animation: edge-selection 1s infinite linear;
}

@keyframes edge-selection {
  to {
    stroke-dashoffset: 6;
  }
}
</style>