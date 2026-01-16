<template>
  <div
    id="minimap"
    class="w-48 h-32 border-2 rounded mb-xs"
    :class="themeClasses('bg-white/90 border-neutral-300', 'bg-black/90 border-neutral-600')"
  >
    <svg
      ref="minimapSvgRef"
      class="w-full h-full"
      :viewBox="`0 0 ${MINIMAP_WIDTH} ${MINIMAP_HEIGHT}`"
      preserveAspectRatio="xMidYMid meet"
      @click="onMinimapClick"
      @mousedown="onMinimapMouseDown"
      @mousemove="onViewportMouseMove"
      @mouseup="onViewportMouseUp"
    >
      <g
        ref="minimapContentRef"
        :transform="`translate(${minimapTransform.translateX}, ${minimapTransform.translateY}) scale(${minimapTransform.scale})`"
      ></g>

      <!-- Viewport indicator - outline box for dragging -->
      <rect
        ref="viewportIndicatorRef"
        :x="viewportBounds?.x ?? 0"
        :y="viewportBounds?.y ?? 0"
        :width="viewportBounds?.width ?? 100"
        :height="viewportBounds?.height ?? 100"
        fill="transparent"
        :stroke="themeClasses('#000000', '#ffffff')"
        stroke-width="1"
        class="viewport-indicator"
        style="cursor: move"
        @mousedown="onViewportMouseDown"
        @click="preventClick"
      />
    </svg>
  </div>
</template>

<script lang="ts" setup>
import { computed, onUnmounted, ref, watch } from "vue";
import { select } from "d3";
import { themeClasses } from "@si/vue-lib/design-system";
import type { layoutNode, GraphData } from "./Map.vue";

type ClusterType = {
  x: number;
  y: number;
  width: number;
  height: number;
  count: number;
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
};

// Props
const props = defineProps<{
  layoutData?: GraphData | null;
  worldBounds: {
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
    width: number;
    height: number;
  };
  viewportCoordinates: { x: number; y: number; width: number; height: number };
  currentScale: number;
}>();

// Emits
const emit = defineEmits<{
  pan: [dx: number, dy: number];
}>();

// Refs
const minimapSvgRef = ref<SVGSVGElement>();
const minimapContentRef = ref<SVGGElement>();
const viewportIndicatorRef = ref<SVGRectElement>();

// Reactive data - fixed minimap coordinate system
const MINIMAP_WIDTH = 192;
const MINIMAP_HEIGHT = 128;

// Calculate transform to fit world bounds in minimap
const minimapTransform = computed(() => {
  const scaleX = MINIMAP_WIDTH / props.worldBounds.width;
  const scaleY = MINIMAP_HEIGHT / props.worldBounds.height;
  const scale = Math.min(scaleX, scaleY) * 0.95; // Leave some padding

  // The key insight: viewport center should map to minimap center
  const viewportCenterX = props.viewportCoordinates.x + props.viewportCoordinates.width / 2;
  const viewportCenterY = props.viewportCoordinates.y + props.viewportCoordinates.height / 2;

  const minimapCenterX = MINIMAP_WIDTH / 2;
  const minimapCenterY = MINIMAP_HEIGHT / 2;

  // Calculate translation so that viewport center maps to minimap center
  const translateX = minimapCenterX - viewportCenterX * scale;
  const translateY = minimapCenterY - viewportCenterY * scale;

  return {
    scale,
    translateX,
    translateY,
  };
});

// Drag state - following Map.vue pattern
const isDragging = ref(false);
const hasDraggedBeyondThreshold = ref(false);
const lastPos = ref<{ x: number; y: number } | null>(null);
const clickWithNoDrag = ref(false);

// Constants
const MAX_MINIMAP_NODES = 200;

// Computed properties
const viewportBounds = computed(() => {
  const transform = minimapTransform.value;

  // Check if viewport coordinates match world bounds (viewing entire world)
  const tolerance = 1; // Small tolerance for floating point comparison
  const viewportMatchesWorld =
    Math.abs(props.viewportCoordinates.x - props.worldBounds.minX) < tolerance &&
    Math.abs(props.viewportCoordinates.y - props.worldBounds.minY) < tolerance &&
    Math.abs(props.viewportCoordinates.width - props.worldBounds.width) < tolerance &&
    Math.abs(props.viewportCoordinates.height - props.worldBounds.height) < tolerance;

  if (viewportMatchesWorld) {
    // When viewing the entire world, viewport indicator should cover entire minimap
    return {
      x: 0,
      y: 0,
      width: MINIMAP_WIDTH,
      height: MINIMAP_HEIGHT,
    };
  }

  // Convert viewport coordinates to minimap coordinate space using transform
  const minimapX = props.viewportCoordinates.x * transform.scale + transform.translateX;
  const minimapY = props.viewportCoordinates.y * transform.scale + transform.translateY;
  const minimapWidth = props.viewportCoordinates.width * transform.scale;
  const minimapHeight = props.viewportCoordinates.height * transform.scale;

  return {
    x: minimapX,
    y: minimapY,
    width: minimapWidth,
    height: minimapHeight,
  };
});

// Core rendering functions
const renderMinimap = () => {
  if (!minimapContentRef.value || !props.layoutData) return;

  const minimapSvg = select(minimapContentRef.value);
  minimapSvg.selectAll("*").remove();

  const children = props.layoutData.children;
  if (!children || children.length === 0) return;

  const shouldSimplifyRender = children.length > MAX_MINIMAP_NODES;

  if (shouldSimplifyRender) {
    renderSimplifiedMinimap(minimapSvg, children);
  } else {
    renderDetailedMinimap(minimapSvg, children);
  }
};

const renderSimplifiedMinimap = (
  minimapSvg: d3.Selection<SVGGElement, unknown, null, undefined>,
  children: layoutNode[],
) => {
  // Calculate adaptive cluster size to achieve exactly 200 elements
  const contentArea = props.worldBounds.width * props.worldBounds.height;
  const targetClusterArea = contentArea / MAX_MINIMAP_NODES;
  const adaptiveClusterSize = Math.sqrt(targetClusterArea);

  const clusters = new Map<string, ClusterType>();

  children.forEach((node) => {
    const clusterX = Math.floor(node.x / adaptiveClusterSize) * adaptiveClusterSize;
    const clusterY = Math.floor(node.y / adaptiveClusterSize) * adaptiveClusterSize;
    const key = `${clusterX}-${clusterY}`;

    if (!clusters.has(key)) {
      clusters.set(key, {
        x: clusterX,
        y: clusterY,
        width: adaptiveClusterSize,
        height: adaptiveClusterSize,
        count: 0,
        minX: node.x,
        minY: node.y,
        maxX: node.x + node.width,
        maxY: node.y + node.height,
      });
    }

    const cluster = clusters.get(key);
    if (!cluster) return;
    cluster.count++;
    // Update cluster bounds to fit actual content
    cluster.minX = Math.min(cluster.minX, node.x);
    cluster.minY = Math.min(cluster.minY, node.y);
    cluster.maxX = Math.max(cluster.maxX, node.x + node.width);
    cluster.maxY = Math.max(cluster.maxY, node.y + node.height);
  });

  // Render all clusters to get as close to 200 elements as possible
  const clustersToRender = Array.from(clusters.values());
  renderClusters(minimapSvg, clustersToRender);
};

const renderIndividualNodes = (
  minimapSvg: d3.Selection<SVGGElement, unknown, null, undefined>,
  children: layoutNode[],
) => {
  minimapSvg
    .selectAll(".minimap-node")
    .data(children)
    .enter()
    .append("rect")
    .attr("class", "minimap-node")
    .attr("x", (d: layoutNode) => d.x)
    .attr("y", (d: layoutNode) => d.y)
    .attr("width", (d: layoutNode) => Math.max(d.width, 2))
    .attr("height", (d: layoutNode) => Math.max(d.height, 2))
    .style("fill", () => {
      const isDark = document.body.classList.contains("dark");
      return isDark ? "#4b5563" : "#6b7280";
    })
    .style("stroke", "none")
    .style("opacity", "0.8");
};

const renderClusters = (minimapSvg: d3.Selection<SVGGElement, unknown, null, undefined>, clusters: ClusterType[]) => {
  const maxCount = Math.max(...clusters.map((c) => c.count));

  minimapSvg
    .selectAll(".minimap-cluster")
    .data(clusters)
    .enter()
    .append("rect")
    .attr("class", "minimap-cluster")
    .attr("x", (d) => d.minX)
    .attr("y", (d) => d.minY)
    .attr("width", (d) => Math.max(d.maxX - d.minX, 3))
    .attr("height", (d) => Math.max(d.maxY - d.minY, 3))
    .style("fill", (d) => {
      const isDark = document.body.classList.contains("dark");
      const opacity = Math.min(0.9, 0.4 + (d.count / maxCount) * 0.5);
      return isDark ? `rgba(75, 85, 99, ${opacity})` : `rgba(107, 114, 128, ${opacity})`;
    })
    .style("stroke", "none");
};

const renderDetailedMinimap = (
  minimapSvg: d3.Selection<SVGGElement, unknown, null, undefined>,
  children: layoutNode[],
) => {
  renderIndividualNodes(minimapSvg, children);
};

// Event handlers
const onMinimapMouseDown = (event: MouseEvent) => {
  // Set up click state for potential click (not drag)
  clickWithNoDrag.value = true;

  // If clicking on viewport indicator, don't handle as general minimap click
  const target = event.target as SVGElement;
  if (target?.classList.contains("viewport-indicator") || target === viewportIndicatorRef.value) {
    return;
  }
};

const onViewportMouseDown = (event: MouseEvent) => {
  event.preventDefault();
  event.stopPropagation();

  isDragging.value = true;
  hasDraggedBeyondThreshold.value = false;
  clickWithNoDrag.value = true;
  lastPos.value = { x: event.clientX, y: event.clientY };

  document.body.style.userSelect = "none";
};

const onViewportMouseMove = (event: MouseEvent) => {
  // If we're dragging the viewport, handle that
  if (isDragging.value && lastPos.value) {
    clickWithNoDrag.value = false;

    // Calculate movement delta like Map.vue does
    const current = { x: event.clientX, y: event.clientY };
    const diff = {
      x: current.x - lastPos.value.x,
      y: current.y - lastPos.value.y,
    };
    lastPos.value = current;

    // Check if we've moved beyond the drag threshold
    if (Math.abs(diff.x) > 1 || Math.abs(diff.y) > 1) {
      hasDraggedBeyondThreshold.value = true;
    }

    const rect = minimapSvgRef.value?.getBoundingClientRect();
    if (!rect) return;

    const transform = minimapTransform.value;

    // Calculate movement scale based on minimap dimensions and transform
    const moveScaleX = MINIMAP_WIDTH / rect.width;
    const moveScaleY = MINIMAP_HEIGHT / rect.height;

    // Calculate movement in minimap SVG coordinates
    const deltaX = diff.x * moveScaleX;
    const deltaY = diff.y * moveScaleY;

    // Convert movement to world coordinates
    const worldDeltaX = deltaX / transform.scale;
    const worldDeltaY = deltaY / transform.scale;

    // Convert to map coordinate space delta (negative because we're moving viewport in opposite direction)
    const mapDx = -worldDeltaX * props.currentScale;
    const mapDy = -worldDeltaY * props.currentScale;

    emit("pan", mapDx, mapDy);
  }
};

const onViewportMouseUp = () => {
  isDragging.value = false;
  document.body.style.userSelect = "";
  lastPos.value = null;

  // Reset drag threshold after a short delay
  setTimeout(() => {
    hasDraggedBeyondThreshold.value = false;
  }, 10);
};

const preventClick = (event: MouseEvent) => {
  // Only prevent click if we actually dragged beyond threshold
  if (hasDraggedBeyondThreshold.value) {
    event.preventDefault();
    event.stopPropagation();
  }
};

const onMinimapClick = (event: MouseEvent) => {
  if (!clickWithNoDrag.value) return;

  const target = event.target as SVGElement;
  if (target?.classList.contains("viewport-indicator") || target === viewportIndicatorRef.value) {
    return;
  }

  const rect = minimapSvgRef.value?.getBoundingClientRect();
  if (!rect) return;

  const clickX = event.clientX - rect.left;
  const clickY = event.clientY - rect.top;
  const minimapSvgX = (clickX / rect.width) * MINIMAP_WIDTH;
  const minimapSvgY = (clickY / rect.height) * MINIMAP_HEIGHT;

  const transform = minimapTransform.value;

  // Convert minimap coordinates to world coordinates
  const worldX = (minimapSvgX - transform.translateX) / transform.scale;
  const worldY = (minimapSvgY - transform.translateY) / transform.scale;

  // Calculate where the viewport center should be to center the clicked point
  const targetViewportCenterX = worldX;
  const targetViewportCenterY = worldY;

  // Calculate the delta from current viewport center to target center
  const currentViewportCenterX = props.viewportCoordinates.x + props.viewportCoordinates.width / 2;
  const currentViewportCenterY = props.viewportCoordinates.y + props.viewportCoordinates.height / 2;

  const layoutDx = targetViewportCenterX - currentViewportCenterX;
  const layoutDy = targetViewportCenterY - currentViewportCenterY;

  // Convert to map coordinate space delta (negative because we're moving viewport in opposite direction)
  const mapDx = -layoutDx * props.currentScale;
  const mapDy = -layoutDy * props.currentScale;

  emit("pan", mapDx, mapDy);
};

// Watchers
watch(
  () => props.layoutData,
  () => {
    renderMinimap();
  },
  { immediate: true, deep: true },
);

// Cleanup on unmount
onUnmounted(() => {
  // Clean up any remaining styles
  document.body.style.userSelect = "";
});

// Expose methods for parent component
defineExpose({
  renderMinimap,
});
</script>

<style lang="less" scoped>
#minimap {
  backdrop-filter: blur(8px);
  transition: opacity 0.2s ease;

  &:hover {
    opacity: 1;
  }

  svg {
    border-radius: 4px;
    cursor: pointer;

    .minimap-node {
      transition: opacity 0.2s ease;

      &:hover {
        opacity: 1 !important;
      }
    }

    .minimap-edge {
      opacity: 0.6;
    }

    .viewport-indicator {
      cursor: move;
      transition: all 0.2s ease;

      &:hover {
        stroke-width: 2;
        opacity: 0.9;
      }

      &:active {
        stroke-width: 2;
        opacity: 1;
      }
    }
  }
}
</style>
