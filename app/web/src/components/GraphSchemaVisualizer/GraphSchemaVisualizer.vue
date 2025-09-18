<template>
  <div class="graph-schema-visualizer h-full w-full relative overflow-hidden">
    <!-- Loading overlay -->
    <div
      v-if="isLayouting"
      class="absolute inset-0 z-10 flex items-center justify-center bg-black bg-opacity-20"
    >
      <div :class="loadingClasses">
        <Icon name="refresh" class="animate-spin" size="lg" />
        <span class="ml-2">Computing layout...</span>
      </div>
    </div>

    <!-- Error message -->
    <div
      v-if="layoutError"
      class="absolute top-4 left-4 right-4 z-10 p-4 rounded-md bg-red-100 border border-red-300 text-red-800"
    >
      Layout Error: {{ layoutError }}
    </div>

    <!-- Controls -->
    <div class="absolute top-4 left-4 z-10 flex flex-col gap-2">
      <!-- Entity type toggles -->
      <!-- <div v-if="availableEntityTypes.length > 1" class="flex flex-wrap gap-2">
        <button
          v-for="entityType in availableEntityTypes"
          :key="entityType"
          :class="entityTypeButtonClasses(entityType)"
          @click="toggleEntityType(entityType)"
        >
          <Icon :name="getEntityIcon(entityType)" size="sm" />
          <span class="ml-1">{{ getEntityLabel(entityType) }}</span>
        </button>
      </div> -->

      <!-- Layout algorithm selector -->
      <select
        v-model="selectedLayoutAlgorithm"
        :class="selectClasses"
        @change="handleLayoutChange"
      >
        <option value="layered">Layered</option>
        <option value="hierarchical">Hierarchical</option>
        <option value="force">Force-directed</option>
      </select>
      
      <!-- Show relationships toggle -->
      <label :class="checkboxLabelClasses">
        <input
          v-model="showRelationships"
          type="checkbox"
          :class="checkboxClasses"
        />
        <span class="ml-2">Show Relationships</span>
      </label>
    </div>

    <!-- Zoom controls -->
    <div class="absolute bottom-4 left-4 z-10 flex flex-col gap-2">
      <button :class="zoomButtonClasses" @click="zoomIn" :disabled="currentScale >= maxZoom">
        <Icon name="plus" size="sm" />
      </button>
      <div :class="zoomDisplayClasses">
        {{ Math.round(currentScale * 100) }}%
      </div>
      <button :class="zoomButtonClasses" @click="zoomOut" :disabled="currentScale <= minZoom">
        <Icon name="minus" size="sm" />
      </button>
      <button :class="zoomButtonClasses" @click="resetView">
        <Icon name="refresh" size="sm" />
      </button>
    </div>

    <!-- Main SVG container -->
    <svg
      ref="svgRef"
      class="w-full h-full"
      @wheel="handleWheel"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseLeave"
    >
      <!-- Define arrow markers -->
      <defs>
        <marker
          v-for="config in arrowMarkerConfigs"
          :key="config.id"
          :id="config.id"
          viewBox="0 0 10 10"
          refX="8"
          refY="3"
          markerWidth="4"
          markerHeight="4"
          orient="auto"
        >
          <path :d="config.path" :fill="config.color" />
        </marker>
      </defs>

      <!-- Main graph group (for pan/zoom transforms) -->
      <g ref="graphGroupRef" :transform="transformString">
        <!-- Edges (render behind nodes) -->
        <g class="edges-layer">
          <EntityEdgeComponent
            v-for="edge in visibleEdges"
            :key="edge.id"
            :edge="edge"
            :sourceNode="getNodeById(edge.sourceId)!"
            :targetNode="getNodeById(edge.targetId)!"
            :isHighlighted="isEdgeHighlighted(edge)"
            :isSelected="selectedEdges.has(edge.id)"
            :showLabel="showRelationships"
            @click="handleEdgeClick"
            @mouseenter="handleEdgeMouseEnter"
            @mouseleave="handleEdgeMouseLeave"
          />
        </g>

        <!-- Nodes -->
        <g class="nodes-layer">
          <EntityNodeComponent
            v-for="node in layoutResult?.nodes || []"
            :key="node.id"
            :node="node"
            :isSelected="selectedNodes.has(node.id)"
            :isHovered="hoveredNode === node.id"
            @contextmenu="handleNodeContextMenu"
            @mouseenter="handleNodeMouseEnter"
            @mouseleave="handleNodeMouseLeave"
            @tableLinkClick="handleTableLinkClick"
          />
        </g>
      </g>

      <!-- Selection rectangle -->
      <rect
        v-if="selectionRect"
        :x="selectionRect.x"
        :y="selectionRect.y"
        :width="selectionRect.width"
        :height="selectionRect.height"
        fill="rgba(59, 130, 246, 0.1)"
        stroke="rgb(59, 130, 246)"
        stroke-width="1"
        stroke-dasharray="4 2"
      />
    </svg>
  </div>
</template>

<script setup lang="ts">
import { 
  ref, 
  computed, 
  watch, 
  onMounted, 
  onUnmounted, 
  nextTick 
} from "vue";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import * as d3 from "d3";
import type { EntityKind } from "@/workers/types/entity_kind_types";
import type { 
  GraphSchemaVisualizerProps,
  Entity,
  EntityNode as EntityNodeType,
  RelationshipEdge,
  GraphViewportState,
  EntitySchemaKind
} from "./types/schema-graph";
import { useEntityGraph } from "./composables/useEntityGraph";
import { useSchemaLayout } from "./composables/useSchemaLayout";
import EntityNodeComponent from "./components/EntityNode.vue";
import EntityEdgeComponent from "./components/EntityEdge.vue";

interface Props extends GraphSchemaVisualizerProps {}

interface Emits {
  (e: "node-click", node: EntityNodeType): void;
  (e: "node-double-click", node: EntityNodeType): void;
  (e: "edge-click", edge: RelationshipEdge): void;
  (e: "selection-change", selectedNodes: Set<string>): void;
  (e: "table-view-request", kind: EntitySchemaKind): void;
}

const props = withDefaults(defineProps<Props>(), {
  showRelationships: true,
  layoutAlgorithm: 'force',
});

const emit = defineEmits<Emits>();

// Refs
const svgRef = ref<SVGSVGElement>();
const graphGroupRef = ref<SVGGElement>();

// Composables
const {
  selectedEntities,
  hoveredEntity,
  createGraphData,
  populateGraph,
  getNodeById: getNodeByIdFromGraph,
  selectEntity,
  deselectEntity,
  clearSelection,
  setHoveredEntity
} = useEntityGraph();

const {
  isLayouting,
  layoutError,
  layoutGraph,
  getNodeAtPosition,
  getViewportForBounds
} = useSchemaLayout();

// Local state
const selectedLayoutAlgorithm = ref(props.layoutAlgorithm || 'layered');
const showRelationships = ref(props.showRelationships || true);
const activeEntityTypes = ref<Set<EntitySchemaKind>>(new Set(props.entityTypes || []));
const selectedNodes = ref<Set<string>>(new Set());
const selectedEdges = ref<Set<string>>(new Set());
const hoveredNode = ref<string | null>(null);
const hoveredEdge = ref<string | null>(null);

// Viewport state
const currentScale = ref(1);
const currentTranslate = ref({ x: 0, y: 0 });
const minZoom = 0.1;
const maxZoom = 3;

// Interaction state
const isDragging = ref(false);
const lastMousePos = ref({ x: 0, y: 0 });
const selectionRect = ref<{ x: number; y: number; width: number; height: number } | null>(null);
const isSelecting = ref(false);

// Layout result
const layoutResult = ref<Awaited<ReturnType<typeof layoutGraph>> | null>(null);

// Available entity types (dynamically determined from entities)
const availableEntityTypes = computed(() => {
 return populateGraph();
});

// Computed properties
const graphData = computed(() => {
  const data = populateGraph();
  return createGraphData(data.entities);
});

const visibleEdges = computed(() => {
  if (!showRelationships.value || !layoutResult.value) return [];
  return layoutResult.value.edges;
});

const transformString = computed(() => {
  return `translate(${currentTranslate.value.x}, ${currentTranslate.value.y}) scale(${currentScale.value})`;
});

// Styling classes
const loadingClasses = computed(() => [
  'flex items-center px-4 py-2 rounded-md text-sm',
  themeClasses('bg-white text-black', 'bg-neutral-800 text-white')
]);

const selectClasses = computed(() => [
  'px-3 py-1 text-sm rounded border',
  themeClasses(
    'bg-white border-neutral-300 text-black',
    'bg-neutral-800 border-neutral-600 text-white'
  )
]);

const checkboxLabelClasses = computed(() => [
  'flex items-center text-sm px-3 py-1 rounded',
  themeClasses('bg-white text-black', 'bg-neutral-800 text-white')
]);

const checkboxClasses = computed(() => [
  'form-checkbox rounded',
  themeClasses('text-action-500', 'text-action-400')
]);

const zoomButtonClasses = computed(() => [
  'p-2 rounded border disabled:opacity-50',
  themeClasses(
    'bg-white border-neutral-300 text-black hover:bg-neutral-50',
    'bg-neutral-800 border-neutral-600 text-white hover:bg-neutral-700'
  )
]);

const zoomDisplayClasses = computed(() => [
  'px-2 py-1 text-xs rounded border text-center',
  themeClasses('bg-white border-neutral-300 text-black', 'bg-neutral-800 border-neutral-600 text-white')
]);

// Theme detection
const isDarkTheme = computed(() => {
  return document.documentElement.classList.contains('dark');
});

// Arrow marker configurations
const arrowMarkerConfigs = computed(() => [
  {
    id: 'arrow-light-normal',
    path: 'M0,0 L0,6 L6,3 z',
    color: isDarkTheme.value ? '#6b7280' : '#9ca3af'
  },
  {
    id: 'arrow-light-highlight', 
    path: 'M0,0 L0,6 L6,3 z',
    color: isDarkTheme.value ? '#60a5fa' : '#3b82f6'
  },
  {
    id: 'arrow-dark-normal',
    path: 'M0,0 L0,6 L6,3 z', 
    color: isDarkTheme.value ? '#6b7280' : '#9ca3af'
  },
  {
    id: 'arrow-dark-highlight',
    path: 'M0,0 L0,6 L6,3 z',
    color: isDarkTheme.value ? '#60a5fa' : '#3b82f6'
  }
]);

// Methods
const entityTypeButtonClasses = (entityType: EntitySchemaKind) => {
  const isActive = activeEntityTypes.value.has(entityType);
  return [
    'flex items-center px-3 py-1 text-sm rounded border transition-colors',
    isActive 
      ? themeClasses('bg-action-200 border-action-500 text-black', 'bg-action-800 border-action-400 text-white')
      : themeClasses('bg-white border-neutral-300 text-black hover:bg-neutral-50', 'bg-neutral-800 border-neutral-600 text-white hover:bg-neutral-700')
  ];
};

const getEntityIcon = (entityType: string): string => {
  switch (entityType) {
    case 'Component': return 'grid';
    case 'SchemaVariant': return 'schematics';
    case 'Function': return 'code';
    default: return 'grid';
  }
};

const getNodeById = (nodeId: string): EntityNodeType | undefined => {
  return layoutResult.value?.nodes.find(node => node.id === nodeId);
};

const isEdgeHighlighted = (edge: RelationshipEdge): boolean => {
  return hoveredEdge.value === edge.id || 
         hoveredNode.value === edge.sourceId || 
         hoveredNode.value === edge.targetId;
};

const toggleEntityType = (entityType: EntitySchemaKind) => {
  if (activeEntityTypes.value.has(entityType)) {
    activeEntityTypes.value.delete(entityType);
  } else {
    activeEntityTypes.value.add(entityType);
  }
  // Trigger reactivity
  activeEntityTypes.value = new Set(activeEntityTypes.value);
};

const handleLayoutChange = () => {
  // Layout will be recomputed due to reactive dependencies
};

const zoomIn = () => {
  const newScale = Math.min(currentScale.value * 1.2, maxZoom);
  setZoom(newScale);
};

const zoomOut = () => {
  const newScale = Math.max(currentScale.value / 1.2, minZoom);
  setZoom(newScale);
};

const setZoom = (scale: number) => {
  currentScale.value = Math.max(minZoom, Math.min(maxZoom, scale));
};

const resetView = () => {
  if (layoutResult.value && svgRef.value) {
    const rect = svgRef.value.getBoundingClientRect();
    const viewport = getViewportForBounds(
      layoutResult.value.bounds,
      rect.width,
      rect.height
    );
    
    currentScale.value = viewport.scale;
    currentTranslate.value = { x: viewport.x, y: viewport.y };
  }
};

// Event handlers
const handleWheel = (event: WheelEvent) => {
  event.preventDefault();
  
  const delta = event.deltaY > 0 ? 0.9 : 1.1;
  const newScale = Math.max(minZoom, Math.min(maxZoom, currentScale.value * delta));
  
  if (newScale !== currentScale.value) {
    const rect = svgRef.value!.getBoundingClientRect();
    const mouseX = event.clientX - rect.left;
    const mouseY = event.clientY - rect.top;
    
    // Zoom towards mouse position
    const factor = newScale / currentScale.value;
    currentTranslate.value.x = mouseX - (mouseX - currentTranslate.value.x) * factor;
    currentTranslate.value.y = mouseY - (mouseY - currentTranslate.value.y) * factor;
    currentScale.value = newScale;
  }
};

const handleMouseDown = (event: MouseEvent) => {
  if (event.button === 0) { // Left mouse button
    isDragging.value = true;
    lastMousePos.value = { x: event.clientX, y: event.clientY };
    
    // Start selection rectangle if holding Shift
    if (event.shiftKey) {
      isSelecting.value = true;
      const rect = svgRef.value!.getBoundingClientRect();
      const x = (event.clientX - rect.left - currentTranslate.value.x) / currentScale.value;
      const y = (event.clientY - rect.top - currentTranslate.value.y) / currentScale.value;
      selectionRect.value = { x, y, width: 0, height: 0 };
    }
  }
};

const handleMouseMove = (event: MouseEvent) => {
  if (isDragging.value) {
    if (isSelecting.value && selectionRect.value) {
      // Update selection rectangle
      const rect = svgRef.value!.getBoundingClientRect();
      const currentX = (event.clientX - rect.left - currentTranslate.value.x) / currentScale.value;
      const currentY = (event.clientY - rect.top - currentTranslate.value.y) / currentScale.value;
      
      selectionRect.value.width = currentX - selectionRect.value.x;
      selectionRect.value.height = currentY - selectionRect.value.y;
    } else {
      // Pan the view
      const dx = event.clientX - lastMousePos.value.x;
      const dy = event.clientY - lastMousePos.value.y;
      
      currentTranslate.value.x += dx;
      currentTranslate.value.y += dy;
      lastMousePos.value = { x: event.clientX, y: event.clientY };
    }
  }
};

const handleMouseUp = (event: MouseEvent) => {
  if (isSelecting.value && selectionRect.value) {
    // Select nodes within rectangle
    const rect = selectionRect.value;
    const minX = Math.min(rect.x, rect.x + rect.width);
    const maxX = Math.max(rect.x, rect.x + rect.width);
    const minY = Math.min(rect.y, rect.y + rect.height);
    const maxY = Math.max(rect.y, rect.y + rect.height);
    
    if (layoutResult.value) {
      layoutResult.value.nodes.forEach(node => {
        const nodeRight = node.position.x + node.dimensions.width;
        const nodeBottom = node.position.y + node.dimensions.height;
        
        if (node.position.x >= minX && nodeRight <= maxX &&
            node.position.y >= minY && nodeBottom <= maxY) {
          selectedNodes.value.add(node.id);
        }
      });
    }
    
    selectionRect.value = null;
    isSelecting.value = false;
  }
  
  isDragging.value = false;
};

const handleMouseLeave = () => {
  isDragging.value = false;
  isSelecting.value = false;
  selectionRect.value = null;
};

const handleNodeClick = (node: EntityNodeType, event: MouseEvent) => {
  if (event.shiftKey) {
    // Multi-select
    if (selectedNodes.value.has(node.id)) {
      selectedNodes.value.delete(node.id);
    } else {
      selectedNodes.value.add(node.id);
    }
  } else {
    // Single select
    selectedNodes.value.clear();
    selectedNodes.value.add(node.id);
  }
  
  emit("node-click", node);
  emit("selection-change", new Set(selectedNodes.value));
};

const handleNodeContextMenu = (node: EntityNodeType, event: MouseEvent) => {
  // Context menu handling would go here
  console.log("Context menu for node:", node.id);
};

const handleNodeMouseEnter = (node: EntityNodeType) => {
  hoveredNode.value = node.id;
};

const handleNodeMouseLeave = (node: EntityNodeType) => {
  hoveredNode.value = null;
};

const handleEdgeClick = (edge: RelationshipEdge, event: MouseEvent) => {
  emit("edge-click", edge);
};

const handleEdgeMouseEnter = (edge: RelationshipEdge) => {
  hoveredEdge.value = edge.id;
};

const handleEdgeMouseLeave = (edge: RelationshipEdge) => {
  hoveredEdge.value = null;
};

const handleTableLinkClick = (node: EntityNodeType) => {
  console.log("Link click");
  emit("table-view-request", node.entityKind);
};

// Watchers
watch([graphData, selectedLayoutAlgorithm], async () => {
  if (graphData.value.nodes.length === 0) {
    layoutResult.value = null;
    return;
  }
  
  try {
    const result = await layoutGraph(graphData.value, {
      algorithm: selectedLayoutAlgorithm.value,
      spacing: {
        nodeNode: 50,
        layerSeparation: 100,
        portPort: 10
      },
      direction: 'DOWN'
    });
    
    layoutResult.value = result;
    
    // Auto-fit view when layout completes
    await nextTick();
    resetView();
  } catch (error) {
    console.error("Layout failed:", error);
  }
}, { immediate: true });

// Lifecycle
onMounted(() => {
  // Any initialization if needed
});

onUnmounted(() => {
  // Cleanup if needed
});
</script>

<style scoped>
.graph-schema-visualizer {
  font-family: system-ui, -apple-system, sans-serif;
}

svg {
  cursor: grab;
}

svg:active {
  cursor: grabbing;
}
</style>