<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ExploreMapSkeleton v-if="showSkeleton" />
  <section
    id="map"
    :class="
      clsx(
        'grid h-full',
        showSkeleton && 'hidden', // Since the svgs need a target to be drawn, we need to have this in the DOM
      )
    "
  >
    <div
      v-if="selectedComponent && selectedComponents.size === 1"
      id="selection"
      :class="
        clsx('absolute top-[110px] w-[435px] right-3', 'flex flex-col gap-xs')
      "
      style="max-height: calc(100vh - 115px)"
    >
      <div
        :class="
          clsx(
            'flex-none p-xs border-2',
            themeClasses(
              'border-action-300 bg-action-200',
              'border-action-700 bg-action-800',
            ),
          )
        "
      >
        <ExploreGridTile
          ref="selectedGridTileRef"
          :component="selectedComponent"
          hideConnections
          @click="navigateToSelectedComponent"
        />
      </div>
      <div class="scrollable grow">
        <ConnectionsPanel :component="selectedComponent" noEmptyState />
      </div>
    </div>

    <div
      id="controls"
      class="absolute left-0 bottom-0 flex flex-col gap-xs m-sm items-start"
    >
      <!-- Minimap -->
      <MiniMap
        v-show="showMinimap"
        :layoutData="dataAsGraph"
        :worldBounds="worldBounds"
        :viewportCoordinates="viewportCoordinates"
        :currentScale="transformMatrix[0] ?? 1"
        @pan="pan"
      />

      <!-- Control buttons -->
      <div class="flex flex-row gap-xs items-center">
        <div
          v-tooltip="showMinimap ? 'Hide Minimap' : 'Show Minimap'"
          :class="getButtonClasses(false)"
          @click="toggleMinimap"
        >
          <Icon name="minimap" size="sm" />
        </div>
        <div
          v-tooltip="'Zoom Out'"
          :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
          @click="zoomOut"
        >
          <Icon name="minus" size="sm" />
        </div>
        <div
          v-tooltip="'Current Zoom'"
          :class="
            clsx(
              'border p-2xs rounded select-none',
              themeClasses(
                'text-black border-black bg-white',
                'text-white border-white bg-black',
              ),
            )
          "
        >
          {{ Math.round(zoomLevel * 100) }}%
        </div>
        <div
          v-tooltip="'Zoom In'"
          :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
          @click="zoomIn"
        >
          <Icon name="plus" size="sm" />
        </div>
        <div
          v-tooltip="'Reset'"
          :class="getButtonClasses(false)"
          @click="reset"
        >
          <Icon name="empty-square" size="sm" />
        </div>
        <div
          v-tooltip="'Help'"
          :class="getButtonClasses(false)"
          @click="emit('help')"
        >
          <Icon name="question-circle" size="sm" />
        </div>
      </div>
    </div>

    <svg
      :class="mouseDown ? 'cursor-grabbing' : 'cursor-grab'"
      height="100%"
      width="100%"
      preserveAspectRatio="xMidYMid"
      :viewBox="`0 0 ${viewBox} ${viewBox}`"
      @wheel="wheel"
      @mousedown.left.prevent="mousedown"
      @mouseup.left="mouseup"
      @mousemove="mousemove"
    >
      <defs v-for="logo in logos" :key="logo">
        <pattern
          :id="`${logo}`"
          width="1"
          height="1"
          patternUnits="objectBoundingBox"
        >
          <IconNoWrapper
            :name="logo"
            :class="themeClasses('text-black', 'text-white')"
          />
        </pattern>
      </defs>
      <template v-for="icon in icons" :key="icon">
        <defs v-for="tone in tones" :key="tone">
          <pattern
            :id="`${icon}-${tone}`"
            width="30"
            height="30"
            patternUnits="objectBoundingBox"
          >
            <IconNoWrapper :name="icon" :tone="tone" size="sm" />
          </pattern>
        </defs>
      </template>

      <!-- Arrow markers for connection directions -->
      <defs>
        <marker
          id="arrowhead"
          markerWidth="10"
          markerHeight="7"
          refX="8"
          refY="3.5"
          orient="auto"
        >
          <path
            d="M 0,0 L 8,3.5 L 0,7"
            fill="none"
            :stroke="themeClasses('#6b7280', '#9ca3af')"
            stroke-width="1.5"
            stroke-linejoin="miter"
          />
        </marker>
        <marker
          id="arrowhead-highlighted"
          markerWidth="10"
          markerHeight="7"
          refX="8"
          refY="3.5"
          orient="auto"
        >
          <path
            d="M 0,0 L 8,3.5 L 0,7"
            fill="none"
            :stroke="themeClasses('#3b82f6', '#93c5fd')"
            stroke-width="1.5"
            stroke-linejoin="miter"
          />
        </marker>
        <marker
          id="arrowhead-greyed"
          markerWidth="10"
          markerHeight="7"
          refX="8"
          refY="3.5"
          orient="auto"
        >
          <path
            d="M 0,0 L 8,3.5 L 0,7"
            fill="none"
            :stroke="themeClasses('#d1d5db', '#4b5563')"
            stroke-width="1.5"
            stroke-linejoin="miter"
            opacity="0.3"
          />
        </marker>
      </defs>
      <g :transform="`matrix(${transformMatrix})`"></g>
      <circle :cx="origCenter" :cy="origCenter" r="5" fill="yellow" />
    </svg>

    <ComponentContextMenu
      ref="componentContextMenuRef"
      onGrid
      @edit="navigateToSelectedComponent"
    />
  </section>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import {
  IconNoWrapper,
  Icon,
  IconNames,
  Tones,
  themeClasses,
  LOGO_ICONS,
} from "@si/vue-lib/design-system";
import {
  computed,
  inject,
  nextTick,
  onMounted,
  onUnmounted,
  reactive,
  ref,
  watch,
} from "vue";
import ELK from "elkjs/lib/elk.bundled.js";
import * as d3 from "d3";
import clsx from "clsx";
import { tw } from "@si/vue-lib";
import { useRoute, useRouter } from "vue-router";
import * as _ from "lodash-es";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  IncomingConnections,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import {
  bifrostList,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import ExploreMapSkeleton from "@/newhotness/skeletons/ExploreMapSkeleton.vue";
import { SelectionsInQueryString } from "./Workspace.vue";
import { KeyDetails } from "./logic_composables/emitters";
import { assertIsDefined, Context, ExploreContext } from "./types";
import ExploreGridTile from "./explore_grid/ExploreGridTile.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";
import { getAssetIcon } from "./util";
import ComponentContextMenu from "./ComponentContextMenu.vue";
import { truncateString } from "./logic_composables/string_funcs";
import MiniMap from "./MiniMap.vue";

const MAX_STRING_LENGTH = 18;

const props = defineProps<{
  active: boolean;
  components: ComponentInList[];
  componentsWithFailedActions: Set<ComponentId>;
}>();

const componentsById = computed<Record<ComponentId, ComponentInList>>(() => {
  return (props.components || []).reduce((obj, component) => {
    obj[component.id] = component;
    return obj;
  }, {} as Record<ComponentId, ComponentInList>);
});

const componentContextMenuRef =
  ref<InstanceType<typeof ComponentContextMenu>>();

const selectedComponents = ref<Set<ComponentInList>>(new Set());
const showMinimap = ref((props.components?.length ?? 0) > 0);

watch(
  () => props.components?.length ?? 0,
  (newLength, oldLength) => {
    if (oldLength === 0 && newLength > 0) {
      showMinimap.value = true;
    } else if (oldLength > 0 && newLength === 0) {
      showMinimap.value = false;
    }
  },
);

// Get the primary selected component (first one in the set)
const selectedComponent = computed<ComponentInList | null>(() => {
  if (selectedComponents.value.size === 0) {
    return null;
  }
  const firstComponent = selectedComponents.value.values().next().value;
  return firstComponent || null;
});

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

const showSkeleton = computed(() => explore.showSkeleton.value);

// don't change this!
// magic number puts the yellow dot in the middle!
// driving me a bit insane
const viewBox = ref(1500);
const origCenter = ref(viewBox.value / 2);
const origTransformMatrix = [1, 0, 0, 1, 0, 0];
const transformMatrix = reactive<number[]>([...origTransformMatrix]);

const scaleStep = 0.13;
const baseZoom = 1;
const zoomLevel = ref(baseZoom);
const lastPos = ref<xy | null>(null);
const MAX_ZOOM = 5;
const MIN_ZOOM = 0.2;

const applyZoom = () => {
  zoomLevel.value = Math.fround(zoomLevel.value);
  zoomLevel.value = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoomLevel.value));

  // Zoom implementation
  // Attempt 1
  // "FPS style" - always zoom to the center point of the SVG
  // move what you want to the center to see it

  // first and fourth elements of the matrix describe
  // the X & Y scale (aka zoom) of the coordinate space
  const scaleX = transformMatrix[0] ?? 1;
  const scaleY = transformMatrix[3] ?? 1;

  // the last two are X & Y adjustments to origin 0,0
  // 0,0 is top left, positive values move down and right
  // negative values move up and left
  // panning the stage changes these values without changing scale
  const translateX = transformMatrix[4] ?? 0;
  const translateY = transformMatrix[5] ?? 0;

  /** find the center of the SVG coordinate space
   * as we scale the coordinate space we need to adjust
   * what "center" is e.g. zooming in, space between points
   * grows which moves the center point down and right
   * and so the top left must move negatively to keep
   * the center point _in_ the center of the screen
   */
  const svgCenterX = origCenter.value;
  const svgCenterY = origCenter.value;

  /** get the difference between "center" and where
   * we have moved the origin off 0,0, from both panning and zoom
   * and divide by the current scale. this normalizes us back to
   * an "unscaled" coordinate space
   */
  const worldX = (svgCenterX - translateX) / scaleX;
  const worldY = (svgCenterY - translateY) / scaleY;

  const newScale = zoomLevel.value;

  // now translate the new points against the new zoom level
  // to get the proper X, Y panning adjustments for the grown/shrunk
  // coordinate space
  const newTranslateX = svgCenterX - worldX * newScale;
  const newTranslateY = svgCenterY - worldY * newScale;

  transformMatrix.splice(
    0,
    6,
    newScale,
    0,
    0,
    newScale,
    newTranslateX,
    newTranslateY,
  );
};

const pan = (dx: number, dy: number) => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const x = transformMatrix[4]!;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const y = transformMatrix[5]!;
  transformMatrix.splice(4, 1, x + dx);
  transformMatrix.splice(5, 1, y + dy);

  // TODO(Wendy) - this fixes the position of the context menu
  // but it is not a perfect fix, as it doesn't handle the
  // selected component going near the edges or offscreen well :(
  if (selectedComponent.value && componentContextMenuRef.value?.isOpen) {
    selectComponent(selectedComponent.value);
  }
};

const mouseDown = ref(false);

// Reactive window dimensions for accurate viewport tracking
const windowDimensions = ref({
  width: window.innerWidth,
  height: window.innerHeight,
});

// Update window dimensions reactively
const updateWindowDimensions = () => {
  windowDimensions.value = {
    width: window.innerWidth,
    height: window.innerHeight,
  };
};

// Calculate world bounds for minimap coordinate system
const worldBounds = computed(() => {
  if (!dataAsGraph.value?.children || dataAsGraph.value.children.length === 0) {
    return {
      minX: -500,
      minY: -500,
      maxX: 500,
      maxY: 500,
      width: 1000,
      height: 1000,
    };
  }

  // Calculate bounding box of all nodes
  let nodeMinX = Infinity;
  let nodeMinY = Infinity;
  let nodeMaxX = -Infinity;
  let nodeMaxY = -Infinity;

  dataAsGraph.value.children.forEach((node: layoutNode) => {
    nodeMinX = Math.min(nodeMinX, node.x);
    nodeMinY = Math.min(nodeMinY, node.y);
    nodeMaxX = Math.max(nodeMaxX, node.x + node.width);
    nodeMaxY = Math.max(nodeMaxY, node.y + node.height);
  });

  // Calculate current viewport area
  const currentScale = transformMatrix[0] ?? 1;
  const translateX = transformMatrix[4] ?? 0;
  const translateY = transformMatrix[5] ?? 0;
  const actualWidth = windowDimensions.value.width;
  const actualHeight = windowDimensions.value.height;

  const viewportWidth = actualWidth / currentScale;
  const viewportHeight = actualHeight / currentScale;
  const viewportX = -translateX / currentScale;
  const viewportY = -translateY / currentScale;

  // Strategy: minimap should always show all nodes prominently for navigation
  // The world bounds should be large enough to include both nodes and viewport,
  // but sized so that nodes remain visible and useful

  // Calculate how much larger the viewport is compared to the node area
  const nodeWidth = nodeMaxX - nodeMinX;
  const nodeHeight = nodeMaxY - nodeMinY;

  // Always include the viewport area
  let finalMinX = Math.min(nodeMinX, viewportX);
  let finalMinY = Math.min(nodeMinY, viewportY);
  let finalMaxX = Math.max(nodeMaxX, viewportX + viewportWidth);
  let finalMaxY = Math.max(nodeMaxY, viewportY + viewportHeight);

  // If the total area is much larger than the node area, constrain it so nodes stay visible
  const totalWidth = finalMaxX - finalMinX;
  const totalHeight = finalMaxY - finalMinY;
  const maxReasonableExpansion = 4; // Don't let the world be more than 4x the node size

  if (totalWidth > nodeWidth * maxReasonableExpansion) {
    const maxWidth = nodeWidth * maxReasonableExpansion;
    const centerX = (finalMinX + finalMaxX) / 2;
    finalMinX = centerX - maxWidth / 2;
    finalMaxX = centerX + maxWidth / 2;
  }

  if (totalHeight > nodeHeight * maxReasonableExpansion) {
    const maxHeight = nodeHeight * maxReasonableExpansion;
    const centerY = (finalMinY + finalMaxY) / 2;
    finalMinY = centerY - maxHeight / 2;
    finalMaxY = centerY + maxHeight / 2;
  }

  // Add small padding for clean edges
  const padding = 20;

  return {
    minX: finalMinX - padding,
    minY: finalMinY - padding,
    maxX: finalMaxX + padding,
    maxY: finalMaxY + padding,
    width: finalMaxX - finalMinX + padding * 2,
    height: finalMaxY - finalMinY + padding * 2,
  };
});

// Calculate viewport coordinates for minimap
const viewportCoordinates = computed(() => {
  const scale = transformMatrix[0] ?? 1;
  const translateX = transformMatrix[4] ?? 0;
  const translateY = transformMatrix[5] ?? 0;

  // Use actual window dimensions for accurate viewport representation
  const actualWidth = windowDimensions.value.width;
  const actualHeight = windowDimensions.value.height;

  // Calculate the viewport area in the main coordinate space
  const viewportWidth = actualWidth / scale;
  const viewportHeight = actualHeight / scale;
  const viewportX = -translateX / scale;
  const viewportY = -translateY / scale;

  return {
    x: viewportX,
    y: viewportY,
    width: viewportWidth,
    height: viewportHeight,
  };
});

const zoomIn = () => {
  zoomLevel.value += scaleStep;
  applyZoom();
};

const zoomOut = () => {
  zoomLevel.value -= scaleStep;
  applyZoom();
};

const wheel = (event: WheelEvent) => {
  if (event.metaKey || event.ctrlKey) {
    // Zoom behavior when modifier key is held
    if (event.deltaY > 0) {
      zoomOut();
    } else {
      zoomIn();
    }
  } else {
    // Pan behavior - move map in scroll direction
    const panSpeed = 40;
    pan((-event.deltaX * panSpeed) / 100, (-event.deltaY * panSpeed) / 100);
  }
};

const reset = () => {
  zoomLevel.value = 1;
  transformMatrix.splice(0, 6, 1, 0, 0, 1, 0, 0);
};

const toggleMinimap = () => {
  showMinimap.value = !showMinimap.value;
};

const clickWithNoDrag = ref(false);

const mousedown = () => {
  mouseDown.value = true;
  clickWithNoDrag.value = true;
};

const mouseup = (e: MouseEvent) => {
  mouseDown.value = false;
  const target = e.target;
  if (target instanceof SVGRectElement && target.classList.contains("node")) {
    // don't deselect if you clicked on a node!
    return;
  } else if (clickWithNoDrag.value) {
    deselect();
    emit("deselect");
  }
};

const active = computed(() => props.active);

const KEYSTEP = 10;
const onArrowDown = () => {
  if (!active.value) return;
  pan(0, -KEYSTEP);
};
const onArrowUp = () => {
  if (!active.value) return;
  pan(0, KEYSTEP);
};
const onArrowLeft = () => {
  if (!active.value) return;
  pan(KEYSTEP, 0);
};
const onArrowRight = () => {
  if (!active.value) return;
  pan(-KEYSTEP, 0);
};
const onEscape = () => {
  if (!active.value) return;

  // If components are selected, deselect them first
  if (selectedComponents.value.size > 0) {
    deselect();
  } else {
    // If no components selected, close map and return to grid
    const query = { ...router.currentRoute.value?.query };
    delete query.map;
    delete query.c;
    query.grid = "1";
    router.push({ query });
  }
};
const onE = (_e: KeyDetails["e"]) => {
  if (selectedComponents.value.size > 0) {
    componentContextMenuRef.value?.componentsStartErase(
      Array.from(selectedComponents.value),
    );
  }
};
const onD = (e: KeyDetails["d"]) => {
  if (selectedComponents.value.size > 0 && (e.metaKey || e.ctrlKey)) {
    const componentIds = Array.from(selectedComponents.value).map(
      (component) => component.id,
    );
    componentContextMenuRef.value?.componentsDuplicate(componentIds);
  }
};
const onP = (_e: KeyDetails["p"]) => {
  // Do nothing! Pinning is unsupported in the map view.
};
const onU = (_e: KeyDetails["u"]) => {
  // if (selectedComponent.value && selectedComponent.value.canBeUpgraded) {
  //   componentContextMenuRef.value?.componentUpgrade([
  //     selectedComponent.value.id,
  //   ]);
  // }
};
const onBackspace = (_e: KeyDetails["Backspace"]) => {
  if (selectedComponents.value.size > 0) {
    const componentsToDelete = Array.from(selectedComponents.value).filter(
      (component) => !component.toDelete,
    );
    if (componentsToDelete.length > 0) {
      componentContextMenuRef.value?.componentsStartDelete(componentsToDelete);
    }
  }
};
const onR = (_e: KeyDetails["r"]) => {
  if (selectedComponents.value.size > 0) {
    const componentsToRestore = Array.from(selectedComponents.value).filter(
      (component) => component.toDelete,
    );
    if (componentsToRestore.length > 0) {
      const componentIds = componentsToRestore.map((component) => component.id);
      componentContextMenuRef.value?.componentsRestore(componentIds);
    }
  }
};
const onM = (_e: KeyDetails["m"]) => {
  toggleMinimap();
};
onMounted(() => {
  // if we need to adjust zoom level on load dynamically
  // change it here
  applyZoom();

  // Set up window resize listener for reactive dimensions
  window.addEventListener("resize", updateWindowDimensions);
});

onUnmounted(() => {
  window.removeEventListener("resize", updateWindowDimensions);
});

const mousemove = (event: MouseEvent) => {
  clickWithNoDrag.value = false;
  const current: xy = { x: event.clientX, y: event.clientY };
  if (!mouseDown.value || !lastPos.value) {
    lastPos.value = { ...current };
    return;
  }
  const diff: xy = {
    x: current.x - lastPos.value.x,
    y: current.y - lastPos.value.y,
  };
  lastPos.value = { ...current };
  pan(diff.x, diff.y);
};

const key = useMakeKey();
const args = useMakeArgs();

const queryKey = key(EntityKind.IncomingConnectionsList);

const logos = reactive<IconNames[]>(Object.keys(LOGO_ICONS) as IconNames[]);

const icons = reactive<IconNames[]>([
  "check-hex-outline",
  "check-hex",
  "x-hex-outline",
]);
const tones = reactive<Tones[]>(["success", "destructive"]);

const connections = useQuery<IncomingConnections[]>({
  queryKey,
  enabled: () => active.value, // Only run query when map view is active
  queryFn: async () => {
    const d = await bifrostList<IncomingConnections[] | null>(
      args(EntityKind.IncomingConnectionsList),
    );

    if (d) {
      // this sets the component from the URL querystring on load, and then doesn't re-enter
      if (fillDefault.value && fillDefault.value.length > 0) {
        nextTick(() => {
          const selectedComps: ComponentInList[] = [];
          fillDefault.value?.forEach((componentId) => {
            const component = componentsById.value[componentId];
            if (component) {
              selectedComps.push(component);
            }
          });

          if (selectedComps.length > 0) {
            selectedComponents.value = new Set(selectedComps);
          }
          fillDefault.value = undefined;
        });
      }

      return d;
    } else return [];
  },
});

const mapData = computed(() => {
  const nodes = new Set<string>();
  const edges = new Set<string>();
  const components: Record<string, ComponentInList> = {};
  if (!connections.data.value || !componentsById.value) {
    return { nodes, edges, components };
  }

  connections.data.value.forEach((c) => {
    if (!componentsById.value[c.id]) return;

    nodes.add(c.id);
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    components[c.id] = componentsById.value[c.id]!;
    c.connections.forEach((e) => {
      // incoming, so "to" is me, always start with "me"
      if (
        !componentsById.value[e.toComponentId] ||
        !componentsById.value[e.fromComponentId]
      )
        return;

      const edge = `${e.toComponentId}-${e.fromComponentId}`;
      edges.add(edge);
    });
  });

  return { nodes, edges, components };
});

export type node = {
  id: string;
  width: number;
  height: number;
  component: ComponentInList;
  icons: [string | null];
};

export type xy = {
  x: number;
  y: number;
};

export type h = {
  $H: number;
};

export type edge = {
  id: string;
  sources: string[];
  targets: string[];
};

export type line = {
  sections: {
    id: string;
    startPoint: xy;
    endPoint: xy;
    bendPoints?: xy[];
    incomingShape: string;
    outgoingShape: string;
  }[];
  container: string;
};

export type layoutNode = node & xy & h;
export type layoutLine = line & edge;

// Type for ELK layout result - simplified to match actual usage
export type GraphData = {
  children: layoutNode[];
  edges: unknown[]; // ELK transforms edges in complex ways
  [key: string]: unknown; // Allow other ELK properties
};

const dataAsGraph = ref<GraphData | null>(null);

const WIDTH = 270;
const HEIGHT = 75;

const selectedGridTileRef = ref<InstanceType<typeof ExploreGridTile>>();

const router = useRouter();
const clickedNode = (e: MouseEvent, n: layoutNode) => {
  e.preventDefault();
  e.stopPropagation();

  if (e.shiftKey) {
    // Multi-select mode with shift-click
    const newSelectedComponents = new Set(selectedComponents.value);

    if (newSelectedComponents.has(n.component)) {
      // Remove from selection if already selected
      newSelectedComponents.delete(n.component);
    } else {
      // Add to selection
      newSelectedComponents.add(n.component);
    }

    selectedComponents.value = newSelectedComponents;

    // Close context menu during multi-select
    if (componentContextMenuRef.value?.isOpen) {
      componentContextMenuRef.value?.close();
    }
  } else {
    // Single select mode
    if (selectedComponent.value?.id === n.component.id) {
      navigateToSelectedComponent();
    } else {
      // Close context menu if it's open before selecting new component
      if (componentContextMenuRef.value?.isOpen) {
        componentContextMenuRef.value?.close();
      }

      // Clear multi-select and set single selection
      selectedComponents.value = new Set([n.component]);
    }
  }
};

const rightClickedNode = (e: MouseEvent, n: layoutNode) => {
  e.preventDefault();
  e.stopPropagation();

  // If the right-clicked component is not in the current selection, select it
  if (!selectedComponents.value.has(n.component)) {
    selectedComponents.value = new Set([n.component]);
  }

  // Show context menu for all selected components
  showContextMenuForSelection(n.component, e.target as Element);
};

const showContextMenuForSelection = (
  anchorComponent: ComponentInList,
  componentEl?: Element,
) => {
  nextTick(() => {
    let element = componentEl;
    if (!element) {
      element = document.getElementsByClassName(`id-${anchorComponent.id}`)[0];
      if (!element) return;
    }

    const rect = element.getBoundingClientRect();

    const anchor = {
      $el: element,
      getBoundingClientRect: () => ({
        ...rect,
        left: rect.right,
        x: rect.right,
        width: 0,
      }),
    };

    // Pass all selected components to the context menu
    const componentsForMenu = Array.from(selectedComponents.value);
    componentContextMenuRef.value?.open(anchor, componentsForMenu);
  });
};

const selectComponent = (component: ComponentInList, componentEl?: Element) => {
  selectedComponents.value = new Set([component]);
  showContextMenuForSelection(component, componentEl);
};
const deselect = () => {
  selectedComponents.value = new Set();
  componentContextMenuRef.value?.close();
};

watch(
  selectedComponents,
  () => {
    // this handles later changes after the page loads
    document.querySelectorAll("#map > svg rect.node").forEach((n) => {
      n.classList.remove("selected");
      n.classList.remove("greyed-out");
      n.classList.remove("failed-actions");
    });

    // Remove greyed-out classes from text elements
    document
      .querySelectorAll("#map > svg text")
      .forEach((n) => n.classList.remove("greyed-out"));

    // Reset opacity for all icons and color bars
    document.querySelectorAll("#map > svg path").forEach((path) => {
      if (path.getAttribute("stroke")) {
        path.setAttribute("stroke-opacity", "1");
      } else {
        (path as HTMLElement).style.opacity = "1";
      }
    });

    // Update edge styling when selection changes
    document.querySelectorAll("#map > svg path.edge").forEach((edge) => {
      const edgeElement = edge as SVGPathElement;
      const edgeId = edgeElement.getAttribute("data-edge-id");

      if (edgeId) {
        const [targetId, sourceId] = edgeId.split("-");
        // Check if edge is connected to any of the selected components
        const isConnectedToSelected = Array.from(selectedComponents.value).some(
          (component) => targetId === component.id || sourceId === component.id,
        );

        if (selectedComponents.value.size > 0) {
          // Update stroke color - theme aware
          const isDark = document.body.classList.contains("dark");
          const connectedColor = isDark ? "#93c5fd" : "#3b82f6"; // action-300 : action-500
          const greyedColor = isDark ? "#4b5563" : "#d1d5db"; // neutral-600 : neutral-300

          edgeElement.style.stroke = isConnectedToSelected
            ? connectedColor
            : greyedColor;
          // Update opacity
          edgeElement.style.opacity = isConnectedToSelected ? "1" : "0.3";
          // Update stroke width
          edgeElement.style.strokeWidth = isConnectedToSelected ? "2" : "1";
          // Update arrow marker
          edgeElement.setAttribute(
            "marker-end",
            isConnectedToSelected
              ? "url(#arrowhead-highlighted)"
              : "url(#arrowhead-greyed)",
          );
        } else {
          // Reset to default when no component is selected - theme aware
          const isDark = document.body.classList.contains("dark");
          const defaultColor = isDark ? "#9ca3af" : "#6b7280"; // neutral-400 : neutral-500

          edgeElement.style.stroke = defaultColor;
          edgeElement.style.opacity = "1";
          edgeElement.style.strokeWidth = "1";
          edgeElement.setAttribute("marker-end", "url(#arrowhead)");
        }
      }
    });

    const query: SelectionsInQueryString = {
      ...router.currentRoute.value?.query,
    };
    delete query.c;

    // Store multiple selected component IDs as comma-separated string
    if (selectedComponents.value.size > 0) {
      const selectedIds = Array.from(selectedComponents.value).map((c) => c.id);
      query.c = selectedIds.join(",");
    }

    // Add selected class to all selected components and ensure they're not greyed out
    selectedComponents.value.forEach((component) => {
      const element = document.querySelector(
        `#map > svg rect.node.id-${component.id}`,
      );
      if (element) {
        element.classList.add("selected");
        element.classList.remove("greyed-out"); // Explicitly remove greyed-out class
      }
    });

    if (selectedComponents.value.size > 0) {
      // Add greyed-out class to unconnected components (except components with no connections at all)
      document.querySelectorAll("#map > svg rect.node").forEach((element) => {
        const componentId = Array.from(element.classList)
          .find((cls) => cls.startsWith("id-"))
          ?.substring(3);

        if (componentId) {
          const isInConnectedIds = connectedComponentIds.value.has(componentId);
          const hasSelectedClass = element.classList.contains("selected");

          if (!isInConnectedIds && !hasSelectedClass) {
            element.classList.add("greyed-out");
          }
        }
      });

      // Add greyed-out class to text elements of unconnected components (except components with no connections at all)
      document.querySelectorAll("#map > svg g").forEach((group) => {
        const rect = group.querySelector("rect.node");
        const componentId = Array.from(rect?.classList || [])
          .find((cls) => cls.startsWith("id-"))
          ?.substring(3);

        if (
          componentId &&
          !connectedComponentIds.value.has(componentId) &&
          !rect?.classList.contains("selected")
        ) {
          // Grey out ALL components that are not connected to selected components
          // when there are selected components (but never grey out selected components)
          group.querySelectorAll("text").forEach((text) => {
            text.classList.add("greyed-out");
          });

          // Grey out icons and color bars
          group.querySelectorAll("path").forEach((path) => {
            if (path.getAttribute("stroke")) {
              // This is likely a color bar
              path.setAttribute("stroke-opacity", "0.3");
            } else {
              // This is likely an icon
              path.style.opacity = "0.3";
            }
          });
        } else if (componentId && rect?.classList.contains("selected")) {
          // If this is a selected component, explicitly remove greyed-out styling
          group.querySelectorAll("text").forEach((text) => {
            text.classList.remove("greyed-out");
          });

          // Reset opacity for icons and color bars
          group.querySelectorAll("path").forEach((path) => {
            if (path.getAttribute("stroke")) {
              path.setAttribute("stroke-opacity", "1");
            } else {
              path.style.opacity = "1";
            }
          });
        }
      });
    }

    // Add failed-actions class to components with failed actions (unless they're selected)
    document.querySelectorAll("#map > svg rect.node").forEach((element) => {
      const componentId = Array.from(element.classList)
        .find((cls) => cls.startsWith("id-"))
        ?.substring(3);

      if (
        componentId &&
        props.componentsWithFailedActions.has(componentId) &&
        !element.classList.contains("selected")
      ) {
        element.classList.add("failed-actions");
      }
    });

    router.push({ query });
  },
  { deep: true },
);

const connectedComponentIds = computed(() => {
  const connectedIds = new Set<string>();
  if (!connections.data.value) {
    return connectedIds;
  }

  // Include all selected components themselves
  selectedComponents.value.forEach((component) => {
    connectedIds.add(component.id);
  });

  // Find all components connected to any of the selected components
  selectedComponents.value.forEach((selectedComp) => {
    const selectedId = selectedComp.id;

    connections.data.value?.forEach((component) => {
      component.connections.forEach((connection) => {
        if (connection.toComponentId === selectedId) {
          connectedIds.add(connection.fromComponentId);
        }
        if (connection.fromComponentId === selectedId) {
          connectedIds.add(connection.toComponentId);
        }
      });
    });
  });

  return connectedIds;
});

const fillDefault = ref<string[]>();
onMounted(() => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  if (query.c) {
    // Parse comma-separated component IDs
    fillDefault.value = query.c.split(",").filter((id) => id.trim());
  }
});

// Watch for when components become available and retry selection if needed
watch(
  [componentsById, fillDefault],
  () => {
    if (
      fillDefault.value &&
      fillDefault.value.length > 0 &&
      Object.keys(componentsById.value).length > 0
    ) {
      nextTick(() => {
        const selectedComps: ComponentInList[] = [];
        fillDefault.value?.forEach((componentId) => {
          const component = componentsById.value[componentId];
          if (component) {
            selectedComps.push(component);
          }
        });

        if (selectedComps.length > 0) {
          selectedComponents.value = new Set(selectedComps);
          fillDefault.value = undefined;
        }
      });
    }
  },
  { immediate: true },
);

// debouncing since the fzf and svg is actually a bit of a grind for every key press
watch(
  mapData,
  _.debounce(async () => {
    // if we filtered away our selection remove it
    if (
      selectedComponent.value?.id &&
      !mapData.value.components[selectedComponent.value.id]
    )
      deselect();

    const children: node[] = [...mapData.value.nodes].map((nId) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const component = mapData.value.components[nId]!;
      const icons: [string | null] = [
        component.hasResource ? "check-hex-success" : null,
      ];
      if (component.qualificationTotals.failed > 0)
        icons.push("x-hex-outline-destructive");
      else icons.push("check-hex-outline-success");
      return { id: nId, width: WIDTH, height: HEIGHT, component, icons };
    });
    const edges: edge[] = [...mapData.value.edges].map((eId) => {
      const [target, source] = eId.split("-");
      return { id: eId, sources: [source ?? ""], targets: [target ?? ""] };
    });
    if (!children || !edges) return null;
    const graph = {
      id: "root",
      layoutOptions: {
        "elk.algorithm": "layered",
        direction: "DOWN",
        spacing: "80",
        "layered.spacing.nodeNodeBetweenLayers": "150",
        "spacing.nodeNode": "80",
        padding: "50",
      },
      children,
      edges,
    };
    const elk = new ELK();
    const layoutedData = await elk.layout(graph);
    // typescript gods help me
    dataAsGraph.value = layoutedData as unknown as GraphData;

    nextTick(() => {
      const svg = d3.select("#map > svg g");
      // the viewbox should be based on "how much of the coordinate space is in use"

      // the types generated aren't exactly matching the actual data!
      // this is what I see
      const children = layoutedData.children as layoutNode[];
      const edges = layoutedData.edges as layoutLine[];

      // clear out for a redraw
      svg.selectAll("*").remove();

      const groups = svg
        .selectAll(".node")
        .data(children)
        .enter()
        .append("g")
        .attr("transform", (d) => `translate(${d.x}, ${d.y})`);

      groups
        .append("rect")
        .attr("width", (d) => d.width)
        .attr("height", (d) => d.height)
        // note this only handles the selected class on load
        .attr("class", (d) => {
          const classes = [`node`, `id-${d.component.id}`];

          if (selectedComponent.value?.id === d.component.id) {
            classes.push("selected");
          } else if (props.componentsWithFailedActions.has(d.component.id)) {
            classes.push("failed-actions");
          }

          return classes.join(" ");
        })
        .on("click", (e: MouseEvent, d: layoutNode) => {
          clickedNode(e, d);
        })
        .on("contextmenu", (e: MouseEvent, d: layoutNode) => {
          rightClickedNode(e, d);
        })
        .on("dblclick", (_e: Event, d: layoutNode) => {
          componentNavigate(d.component.id);
        });

      groups
        .append("path")
        .attr("d", () => {
          const lineGenerator = d3.line();
          return lineGenerator([
            [0, 0],
            [0, HEIGHT],
          ]);
        })
        .attr("stroke", (d) => d.component.color ?? "#111111")
        .attr("stroke-width", 3)
        .attr("pointer-events", "none"); // prevents this from being clickable

      // logos
      groups
        .append("path")
        .attr("d", d3.symbol().size(1000).type(d3.symbolSquare))
        .attr("transform", () => {
          return "translate(23, 35)";
        })
        .attr("pointer-events", "none") // prevents this from being clickable
        .style("fill", (d) => {
          const icon = getAssetIcon(d.component.schemaCategory);
          return `url(#${icon})`;
        });

      groups
        .append("text")
        .text((d) => truncateString(d.component.name, MAX_STRING_LENGTH))
        .attr("dx", "45")
        .attr("dy", "25")
        .attr("class", "name")
        .attr("alignment-baseline", "middle")
        .attr("pointer-events", "none"); // prevents this from being clickable

      groups
        .append("text")
        .text((d) =>
          truncateString(d.component.schemaVariantName, MAX_STRING_LENGTH),
        )
        .attr("dx", "45")
        .attr("dy", "45")
        .attr("class", "")
        .attr("alignment-baseline", "middle")
        .attr("color", "white")
        .attr("pointer-events", "none"); // prevents this from being clickable

      // qual & resource icons
      groups.each(function doTheIcons(d) {
        d.icons.forEach((icon, idx) => {
          d3.select(this)
            .append("path")
            // i have zero idea what this size param does
            // smaller value the SVG gets bigger and blurry
            // 1000 value its "correct" & sharp
            // even larger and it starts moving on
            .attr("d", d3.symbol().size(1000).type(d3.symbolSquare))
            .attr("transform", () => {
              return `translate(${WIDTH - 13 - 23 * idx}, ${HEIGHT - 9})`;
            })
            .attr("pointer-events", "none")
            .style("fill", () => {
              return icon ? `url(#${icon})` : "none";
            });
        });
      });

      svg
        .selectAll(".edge")
        .data(edges)
        .enter()
        .append("path")
        .attr("class", (d) => {
          const [targetId, sourceId] = d.id.split("-");
          const isConnectedToSelected =
            selectedComponent.value &&
            (targetId === selectedComponent.value.id ||
              sourceId === selectedComponent.value.id);
          return `edge ${isConnectedToSelected ? "connected" : ""}`;
        })
        .attr("data-edge-id", (d) => d.id)
        .attr("d", (d) => {
          const lineGenerator = d3.line();

          const points: xy[] = [];
          d.sections.forEach((section) => {
            points.push({ x: section.startPoint.x, y: section.startPoint.y });
            if (section.bendPoints) points.push(...section.bendPoints);
            points.push({ x: section.endPoint.x, y: section.endPoint.y });
          });
          const pairs: Array<[number, number]> = points.map((p) => [p.x, p.y]);
          return lineGenerator(pairs);
        })
        .attr("marker-end", (d) => {
          const [targetId, sourceId] = d.id.split("-");
          const isConnectedToSelected =
            selectedComponent.value &&
            (targetId === selectedComponent.value.id ||
              sourceId === selectedComponent.value.id);

          if (selectedComponent.value) {
            return isConnectedToSelected
              ? "url(#arrowhead-highlighted)"
              : "url(#arrowhead-greyed)";
          } else {
            return "url(#arrowhead)";
          }
        })
        .style("fill", "none")
        .style("stroke", (d) => {
          const [targetId, sourceId] = d.id.split("-");
          const isConnectedToSelected =
            selectedComponent.value &&
            (targetId === selectedComponent.value.id ||
              sourceId === selectedComponent.value.id);

          // Theme-aware colors
          const isDark = document.body.classList.contains("dark");
          const connectedColor = isDark ? "#93c5fd" : "#3b82f6"; // action-300 : action-500
          const greyedColor = isDark ? "#4b5563" : "#d1d5db"; // neutral-600 : neutral-300
          const defaultColor = isDark ? "#9ca3af" : "#6b7280"; // neutral-400 : neutral-500

          if (selectedComponent.value) {
            return isConnectedToSelected ? connectedColor : greyedColor;
          } else {
            return defaultColor;
          }
        })
        .style("stroke-width", (d) => {
          const [targetId, sourceId] = d.id.split("-");
          const isConnectedToSelected =
            selectedComponent.value &&
            (targetId === selectedComponent.value.id ||
              sourceId === selectedComponent.value.id);
          return isConnectedToSelected ? "2" : "1";
        })
        .style("opacity", (d) => {
          const [targetId, sourceId] = d.id.split("-");
          const isConnectedToSelected =
            selectedComponent.value &&
            (targetId === selectedComponent.value.id ||
              sourceId === selectedComponent.value.id);

          if (selectedComponent.value) {
            return isConnectedToSelected ? "1" : "0.3"; // Reduce opacity for unconnected lines
          } else {
            return "1"; // Full opacity when no component is selected
          }
        });
    });
    // Don't show context menu when component is selected via URL parameter
    // Context menu should only show on user interaction (right-click)
  }, 100),
  { immediate: true },
);

function getButtonClasses(isDisabled: boolean) {
  return clsx(
    tw`rounded-full p-1 border`,
    themeClasses(
      "bg-neutral-600 text-white active:bg-neutral-200 hover:bg-neutral-400 active:text-black active:border-black",
      "bg-neutral-200 text-black active:bg-neutral-700 hover:bg-neutral-400 active:text-white active:border-white",
    ),
    isDisabled ? tw`cursor-not-allowed opacity-50` : tw`cursor-pointer`,
  );
}

const route = useRoute();
const componentNavigate = (componentId: ComponentId) => {
  const params = { ...route.params };
  params.componentId = componentId;
  router.push({
    name: "new-hotness-component",
    params,
    query: {},
  });
};

const navigateToSelectedComponent = () => {
  if (selectedComponent.value?.id) {
    componentNavigate(selectedComponent.value.id);
  }
};

// TODO there's a noticeable time before drawing svgs that we should take into account for setting this flag
const isLoading = computed(() => connections.isLoading.value);

const emit = defineEmits<{
  (e: "deselect"): void;
  (e: "help"): void;
}>();

defineExpose({
  deselect,
  navigateToSelectedComponent,
  onArrowUp,
  onArrowDown,
  onArrowLeft,
  onArrowRight,
  onEscape,
  onE,
  onD,
  onP,
  onU,
  onBackspace,
  onR,
  onM,
  isLoading,
});
</script>

<style lang="less">
#map > svg {
  rect.node {
    fill: @colors-neutral-100;
    stroke: @colors-neutral-500;
    stroke-width: 2;

    body.dark & {
      fill: @colors-neutral-800;
    }

    &.selected {
      stroke: @colors-action-500;
      stroke-width: 3;
      body.dark & {
        stroke: @colors-action-300;
      }
    }

    &.failed-actions {
      stroke: @colors-destructive-600;
      stroke-width: 2;
      body.dark & {
        stroke: @colors-destructive-400;
      }
    }

    &:hover {
      fill: @colors-neutral-200;
      body.dark & {
        fill: @colors-neutral-700;
      }
      cursor: pointer;
    }

    &.selected:hover {
      fill: @colors-neutral-200;
      body.dark & {
        fill: @colors-neutral-700;
      }
    }

    &.greyed-out {
      fill: @colors-neutral-200;
      stroke: @colors-neutral-300;
      opacity: 0.3;

      body.dark & {
        fill: @colors-neutral-700;
        stroke: @colors-neutral-600;
      }
    }
  }

  text {
    fill: black;
    body.dark & {
      fill: white;
    }
    font-size: 1rem;
    &.name {
      font-weight: bold;
      font-size: 1.2rem;
    }

    &.greyed-out {
      fill: @colors-neutral-400;
      opacity: 0.6;

      body.dark & {
        fill: @colors-neutral-500;
      }
    }
  }

  path.edge {
    stroke: @colors-neutral-500;
    stroke-width: 1;

    body.dark & {
      stroke: @colors-neutral-400;
    }

    &.connected {
      stroke: @colors-action-500;
      stroke-width: 2;

      body.dark & {
        stroke: @colors-action-300;
      }
    }
  }
}
</style>
