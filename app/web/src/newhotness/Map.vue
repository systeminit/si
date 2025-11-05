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
            'flex-none p-xs',
            selectedComponentHasFailedActions
              ? themeClasses('bg-destructive-100', 'bg-destructive-900')
              : themeClasses('bg-action-200', 'bg-action-800'),
          )
        "
        :style="{
          border: selectedComponentHasFailedActions
            ? '0.5px solid rgb(239 68 68)' // destructive-500
            : '0.5px solid rgb(59 130 246)', // action-500
        }"
      >
        <ExploreGridTile
          ref="selectedGridTileRef"
          :component="selectedComponent!"
          hideConnections
          @click="navigateToSelectedComponent"
        />
      </div>
      <div class="scrollable grow">
        <ConnectionsPanel :component="selectedComponent!" inMap />
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
            :fillColor="theme === 'light' ? 'black' : 'white'"
            :forcedSizeNumbers="getLogoForcedSizeNumbers(logo)"
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
    </svg>

    <ComponentContextMenu
      ref="componentContextMenuRef"
      onGrid
      hidePin
      hideBulk
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
  useTheme,
  LOGO_FORCED_SIZE_NUMBERS,
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
import { pickBrandIconByString } from "./util";
import ComponentContextMenu from "./ComponentContextMenu.vue";
import { truncateString } from "./logic_composables/string_funcs";
import MiniMap from "./MiniMap.vue";

const MAX_STRING_LENGTH = 18;

const router = useRouter();
const { theme } = useTheme();

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
// Store selected component IDs to maintain selection across component updates
const selectedComponentIds = computed(
  () => new Set(Array.from(selectedComponents.value).map((c) => c.id)),
);

// Watch for componentsById changes and update selectedComponents with new object references
watch(componentsById, (newComponentsById) => {
  if (selectedComponentIds.value.size > 0) {
    const newSelectedComponents = new Set<ComponentInList>();
    selectedComponentIds.value.forEach((componentId) => {
      const component = newComponentsById[componentId];
      if (component) {
        newSelectedComponents.add(component);
      }
    });
    selectedComponents.value = newSelectedComponents;
    emit("selectedComponents", selectedComponents.value);
  }
});

const showMinimap = ref((props.components?.length ?? 0) > 0);

watch(
  () => props.components?.length ?? 0,
  (newLength, oldLength) => {
    const isHideUnconnected =
      router.currentRoute.value.query.hideSubscriptions === "1";

    if (oldLength === 0 && newLength > 0) {
      // Only show minimap if not in hideSubscriptions mode
      showMinimap.value = !isHideUnconnected;
    } else if (oldLength > 0 && newLength === 0) {
      showMinimap.value = false;
    }
  },
);

// Hide minimap when in hideSubscriptions mode
watch(
  () => router.currentRoute.value.query.hideSubscriptions,
  (hideSubscriptions) => {
    if (hideSubscriptions === "1") {
      showMinimap.value = false;
    } else {
      // Restore minimap if we have components
      showMinimap.value = (props.components?.length ?? 0) > 0;
    }
  },
  { immediate: true },
);

// Get the primary selected component (first one in the set)
const selectedComponent = computed<ComponentInList | null>(() => {
  if (selectedComponents.value.size === 0) {
    return null;
  }
  const firstComponent = selectedComponents.value.values().next().value;
  return firstComponent || null;
});

// Check if selected component has failed actions
const selectedComponentHasFailedActions = computed(() => {
  return (
    selectedComponent.value &&
    props.componentsWithFailedActions.has(selectedComponent.value.id)
  );
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

const smoothPan = (totalDx: number, totalDy: number, duration = 400) => {
  const startTime = performance.now();
  const startTranslateX = transformMatrix[4] ?? 0;
  const startTranslateY = transformMatrix[5] ?? 0;

  const animate = (currentTime: number) => {
    const elapsed = currentTime - startTime;
    const progress = Math.min(elapsed / duration, 1);

    // Ease-out cubic for smooth deceleration
    const easeProgress = 1 - (1 - progress) ** 3;

    // Calculate current target position
    const currentX = startTranslateX + totalDx * easeProgress;
    const currentY = startTranslateY + totalDy * easeProgress;

    // Set the transform matrix directly
    transformMatrix.splice(4, 1, currentX);
    transformMatrix.splice(5, 1, currentY);

    if (progress < 1) {
      requestAnimationFrame(animate);
    }
  };

  requestAnimationFrame(animate);
};

// Viewport preservation functions
const preserveViewportCenter = () => {
  const scale = transformMatrix[0] ?? 1;
  const translateX = transformMatrix[4] ?? 0;
  const translateY = transformMatrix[5] ?? 0;

  return {
    worldCenterX: (windowDimensions.value.width / 2 - translateX) / scale,
    worldCenterY: (windowDimensions.value.height / 2 - translateY) / scale,
    scale,
  };
};

const restoreViewportCenter = (
  snapshot: ReturnType<typeof preserveViewportCenter>,
  layoutData: GraphData | null,
) => {
  if (!layoutData?.children) return;

  const scale = transformMatrix[0] ?? 1;
  const newTranslateX =
    windowDimensions.value.width / 2 - snapshot.worldCenterX * scale;
  const newTranslateY =
    windowDimensions.value.height / 2 - snapshot.worldCenterY * scale;

  transformMatrix.splice(4, 1, newTranslateX);
  transformMatrix.splice(5, 1, newTranslateY);
};

// Helper functions for creating node elements and edges
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const addNodeElements = (group: any, d: layoutNode) => {
  group
    .append("rect")
    .attr("width", d.width)
    .attr("height", d.height)
    .attr("class", () => {
      const classes = [`node`, `id-${d.component.id}`];

      // Check if this component is in the current selection
      const isSelected = Array.from(selectedComponents.value).some(
        (c) => c.id === d.component.id,
      );
      if (isSelected) {
        classes.push("selected");
      }
      if (props.componentsWithFailedActions.has(d.component.id)) {
        classes.push("failed-actions");
      }

      return classes.join(" ");
    })
    .on("click", (e: MouseEvent) => {
      clickedNode(e, d);
    })
    .on("contextmenu", (e: MouseEvent) => {
      rightClickedNode(e, d);
    })
    .on("dblclick", (_e: Event) => {
      componentNavigate(d.component.id);
    });

  group
    .append("path")
    .attr("d", () => {
      const lineGenerator = d3.line();
      return lineGenerator([
        [0, 0],
        [0, HEIGHT],
      ]);
    })
    .attr("stroke", d.component.color ?? "#111111")
    .attr("stroke-width", 3)
    .attr("pointer-events", "none");

  // logos
  group
    .append("path")
    .attr("d", d3.symbol().size(1000).type(d3.symbolSquare))
    .attr("transform", "translate(23, 35)")
    .attr("pointer-events", "none")
    .style("fill", () => {
      const icon = pickBrandIconByString(d.component.schemaCategory);
      return `url(#${icon})`;
    });

  group
    .append("text")
    .text(truncateString(d.component.name, MAX_STRING_LENGTH))
    .attr("dx", "45")
    .attr("dy", "25")
    .attr("class", "name")
    .attr("alignment-baseline", "middle")
    .attr("pointer-events", "none");

  group
    .append("text")
    .text(truncateString(d.component.schemaVariantName, MAX_STRING_LENGTH))
    .attr("dx", "45")
    .attr("dy", "45")
    .attr("class", "")
    .attr("alignment-baseline", "middle")
    .attr("color", "white")
    .attr("pointer-events", "none");

  // qual & resource icons
  d.icons.forEach((icon: string | null, idx: number) => {
    group
      .append("path")
      .attr("d", d3.symbol().size(1000).type(d3.symbolSquare))
      .attr("transform", `translate(${WIDTH - 13 - 23 * idx}, ${HEIGHT - 9})`)
      .attr("pointer-events", "none")
      .style("fill", icon ? `url(#${icon})` : "none");
  });

  // Socket connections alert icon
  if (d.component.hasSocketConnections) {
    group
      .append("path")
      .attr("d", d3.symbol().size(1000).type(d3.symbolSquare))
      .attr("transform", () => {
        const iconOffset = d.icons.length * 23;
        return `translate(${WIDTH - 13 - iconOffset - 23}, ${HEIGHT - 9})`;
      })
      .attr("pointer-events", "none")
      .style("fill", "url(#alert-triangle-filled-warning)");
  }
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const renderEdges = (svg: any, edges: layoutLine[]) => {
  svg
    .selectAll(".edge")
    .data(edges)
    .enter()
    .append("path")
    .attr("class", (d: layoutLine) => {
      const [targetId, sourceId] = d.id.split("-");
      const isConnectedToSelected = Array.from(selectedComponents.value).some(
        (component) => targetId === component.id || sourceId === component.id,
      );
      return `edge ${isConnectedToSelected ? "connected" : ""}`;
    })
    .attr("data-edge-id", (d: layoutLine) => d.id)
    .attr("d", (d: layoutLine) => {
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
    .attr("marker-end", (d: layoutLine) => {
      const [targetId, sourceId] = d.id.split("-");
      const isConnectedToSelected = Array.from(selectedComponents.value).some(
        (component) => targetId === component.id || sourceId === component.id,
      );

      if (selectedComponents.value.size > 0) {
        return isConnectedToSelected
          ? "url(#arrowhead-highlighted)"
          : "url(#arrowhead-greyed)";
      } else {
        return "url(#arrowhead)";
      }
    })
    .style("fill", "none")
    .style("stroke", (d: layoutLine) => {
      const [targetId, sourceId] = d.id.split("-");
      const isConnectedToSelected = Array.from(selectedComponents.value).some(
        (component) => targetId === component.id || sourceId === component.id,
      );

      const isDark = document.body.classList.contains("dark");
      const connectedColor = isDark ? "#93c5fd" : "#3b82f6";
      const greyedColor = isDark ? "#4b5563" : "#d1d5db";
      const defaultColor = isDark ? "#9ca3af" : "#6b7280";

      if (selectedComponents.value.size > 0) {
        return isConnectedToSelected ? connectedColor : greyedColor;
      } else {
        return defaultColor;
      }
    })
    .style("stroke-width", (d: layoutLine) => {
      const [targetId, sourceId] = d.id.split("-");
      const isConnectedToSelected = Array.from(selectedComponents.value).some(
        (component) => targetId === component.id || sourceId === component.id,
      );
      return isConnectedToSelected ? "2" : "1";
    })
    .style("opacity", (d: layoutLine) => {
      const [targetId, sourceId] = d.id.split("-");
      const isConnectedToSelected = Array.from(selectedComponents.value).some(
        (component) => targetId === component.id || sourceId === component.id,
      );

      if (selectedComponents.value.size > 0) {
        return isConnectedToSelected ? "1" : "0.3";
      } else {
        return "1";
      }
    });
};

// Function to ensure selection state is properly applied to DOM elements
const applySelectionState = () => {
  // Clear all selection classes first
  const allNodes = document.querySelectorAll("#map > svg rect.node");
  allNodes.forEach((element) => {
    element.classList.remove("selected");
  });

  // Apply selection to currently selected components
  selectedComponents.value.forEach((component) => {
    const element = document.querySelector(
      `#map > svg rect.node.id-${component.id}`,
    );
    if (element) {
      element.classList.add("selected");
    }
  });

  // Update failed action classes
  document.querySelectorAll("#map > svg rect.node").forEach((element) => {
    const componentId = Array.from(element.classList)
      .find((cls) => cls.startsWith("id-"))
      ?.substring(3);

    if (componentId) {
      const hasFailedActions =
        props.componentsWithFailedActions.has(componentId);
      if (hasFailedActions) {
        element.classList.add("failed-actions");
      } else {
        element.classList.remove("failed-actions");
      }
    }
  });
};

const panToComponent = (componentId: string) => {
  if (!dataAsGraph.value?.children) {
    // Layout not ready yet, retry after a short delay
    setTimeout(() => panToComponent(componentId), 100);
    return;
  }

  // Find the node for this component
  const node = dataAsGraph.value.children.find(
    (n: layoutNode) => n.component.id === componentId,
  );
  if (!node) {
    // Node not found in layout, retry after a short delay
    setTimeout(() => panToComponent(componentId), 100);
    return;
  }

  // Calculate the center of the component node
  const componentCenterX = node.x + node.width / 2;
  const componentCenterY = node.y + node.height / 2;

  // Calculate the center of the viewport
  const viewportCenterX = windowDimensions.value.width / 2;
  const viewportCenterY = windowDimensions.value.height / 2;

  // Get current scale and transform
  const scale = transformMatrix[0] ?? 1;
  const currentTranslateX = transformMatrix[4] ?? 0;
  const currentTranslateY = transformMatrix[5] ?? 0;

  // Calculate where the component center currently appears on screen
  const currentScreenX = componentCenterX * scale + currentTranslateX;
  const currentScreenY = componentCenterY * scale + currentTranslateY;

  // Calculate how much we need to pan to center the component
  const panDx = viewportCenterX - currentScreenX;
  const panDy = viewportCenterY - currentScreenY;

  // Use smooth animated pan
  smoothPan(panDx, panDy);
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
    delete query.hideSubscriptions; // Also clear hideSubscriptions when closing map
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
    componentContextMenuRef.value?.duplicateComponentStart(componentIds);
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

const getLogoForcedSizeNumbers = (logoName: IconNames) => {
  return LOGO_FORCED_SIZE_NUMBERS[logoName] ?? undefined;
};

const icons = reactive<IconNames[]>([
  "check-hex-outline",
  "check-hex",
  "x-hex-outline",
  "alert-triangle-filled",
]);
const tones = reactive<Tones[]>(["success", "destructive", "warning"]);

const connections = useQuery<IncomingConnections[]>({
  queryKey,
  enabled: () => active.value, // Only run query when map view is active
  queryFn: async () => {
    const d = await bifrostList<IncomingConnections[] | null>(
      args(EntityKind.IncomingConnectionsList),
    );
    return d ?? [];
  },
});

const mapData = computed(() => {
  const nodes = new Set<string>();
  const edges = new Set<string>();
  const components: Record<string, ComponentInList> = {};
  if (!connections.data.value || !componentsById.value) {
    return { nodes, edges, components };
  }

  // Check if we should filter to only show connected components
  const shouldHideUnconnected =
    router.currentRoute.value.query.hideSubscriptions === "1";

  const hasSelectedComponents = selectedComponents.value.size > 0;

  // First pass: collect all components and their connections
  const allComponents = new Map<string, ComponentInList>();
  const allConnections = new Set<string>();

  connections.data.value.forEach((c) => {
    const component = componentsById.value[c.id];
    if (!component) return;
    allComponents.set(c.id, component);

    c.connections.forEach((e) => {
      if (
        !componentsById.value[e.toComponentId] ||
        !componentsById.value[e.fromComponentId]
      )
        return;

      const edge = `${e.toComponentId}-${e.fromComponentId}`;
      allConnections.add(edge);
    });
  });

  // If we're in hideSubscriptions mode and have selected components, filter the data
  if (shouldHideUnconnected && hasSelectedComponents) {
    // Get the connected component IDs (including selected ones)
    const connectedIds = new Set<string>();

    // Always include selected components
    selectedComponents.value.forEach((comp) => {
      connectedIds.add(comp.id);
    });

    // Add directly connected components
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

    // Only include connected components in the final data
    connectedIds.forEach((componentId) => {
      const component = allComponents.get(componentId);
      if (component) {
        nodes.add(componentId);
        components[componentId] = component;
      }
    });

    // Only include edges between visible components
    allConnections.forEach((edgeId) => {
      const [targetId, sourceId] = edgeId.split("-");
      if (
        targetId &&
        sourceId &&
        connectedIds.has(targetId) &&
        connectedIds.has(sourceId)
      ) {
        edges.add(edgeId);
      }
    });
  } else {
    // Normal mode: include all components
    allComponents.forEach((component, componentId) => {
      nodes.add(componentId);
      components[componentId] = component;
    });

    allConnections.forEach((edge) => {
      edges.add(edge);
    });
  }

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
const previousLayout = ref<GraphData | null>(null);

const WIDTH = 270;
const HEIGHT = 75;

const selectedGridTileRef = ref<InstanceType<typeof ExploreGridTile>>();

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
    // Update selectedComponentIds to maintain stable selection

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

      // Pan to center the selected component
      panToComponent(n.component.id);
    }
  }
  emit("selectedComponents", selectedComponents.value);
};

const rightClickedNode = (e: MouseEvent, n: layoutNode) => {
  e.preventDefault();
  e.stopPropagation();

  // If the right-clicked component is not in the current selection, add it to selection
  if (!selectedComponents.value.has(n.component)) {
    const newSelectedComponents = new Set(selectedComponents.value);
    newSelectedComponents.add(n.component);
    selectedComponents.value = newSelectedComponents;
    emit("selectedComponents", selectedComponents.value);
  }

  // Show context menu for all selected components
  // wait for the map to animate
  setTimeout(() => {
    showContextMenuForSelection(n.component, e.target as Element);
  }, 500);
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
  emit("selectedComponents", selectedComponents.value);
};
const deselect = () => {
  selectedComponents.value = new Set();
  componentContextMenuRef.value?.close();
  emit("selectedComponents", selectedComponents.value);
};

// Note: Connection filtering is now handled in the mapData computed,
// so we no longer need separate DOM manipulation for hiding/showing components

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

// Clean up hideSubscriptions parameter when leaving the map
onUnmounted(() => {
  const currentQuery = router.currentRoute.value.query;
  if (currentQuery.hideSubscriptions === "1") {
    const newQuery = { ...currentQuery };
    delete newQuery.hideSubscriptions;
    router.replace({ query: newQuery });
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
          emit("selectedComponents", selectedComponents.value);
          // Set up pending pan for when layout becomes available
          if (selectedComps[0]) {
            pendingPanComponent.value = selectedComps[0].id;
          }
          fillDefault.value = undefined;
        }
      });
    }
  },
  { immediate: true },
);

// Watch for when layout data becomes available and pan to selected component from URL
const pendingPanComponent = ref<string | null>(null);

watch(
  [dataAsGraph, selectedComponents],
  () => {
    // If we have a pending pan request and the layout is now available
    if (
      pendingPanComponent.value &&
      dataAsGraph.value?.children &&
      selectedComponents.value.size > 0
    ) {
      const componentId = pendingPanComponent.value;
      const node = dataAsGraph.value.children.find(
        (n: layoutNode) => n.component.id === componentId,
      );

      if (node) {
        // Use nextTick to ensure DOM is updated
        nextTick(() => {
          panToComponent(componentId);
          pendingPanComponent.value = null;
        });
      }
    }
  },
  { immediate: true },
);

// when the change set changes, we dont want to animate, we want to re-draw entirely.
watch(ctx.changeSetId, () => {
  previousLayout.value = null;
});

// debouncing since the fzf and svg is actually a bit of a grind for every key press
// PSA: selecting a component re-fires this watcher. Why? Entirely unsure.
watch(
  mapData,
  _.debounce(
    async () => {
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

      const validNodeIds = new Set(mapData.value.nodes);
      const edges: edge[] = [...mapData.value.edges]
        .map((eId) => {
          const [target, source] = eId.split("-");
          return { id: eId, sources: [source ?? ""], targets: [target ?? ""] };
        })
        .filter((edge) => {
          // Only include edges where both source and target exist in the nodes
          const source = edge.sources[0];
          const target = edge.targets[0];
          const sourceExists = source && validNodeIds.has(source);
          const targetExists = target && validNodeIds.has(target);
          return sourceExists && targetExists;
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
      // Preserve current viewport center before layout change
      const viewportSnapshot = preserveViewportCenter();

      const elk = new ELK();
      const layoutedData = await elk.layout(graph);
      // typescript gods help me
      dataAsGraph.value = layoutedData as unknown as GraphData;

      // Restore viewport to maintain visual continuity
      if (previousLayout.value && dataAsGraph.value) {
        restoreViewportCenter(viewportSnapshot, dataAsGraph.value);
      }

      nextTick(() => {
        const svg = d3.select("#map > svg g");
        // the viewbox should be based on "how much of the coordinate space is in use"

        // the types generated aren't exactly matching the actual data!
        // this is what I see
        const children = layoutedData.children as layoutNode[];
        const edges = layoutedData.edges as layoutLine[];

        // Check if we should animate or do immediate update
        const shouldAnimate =
          previousLayout.value && previousLayout.value.children;

        if (shouldAnimate && previousLayout.value) {
          // Animate existing nodes to new positions
          const oldNodes = previousLayout.value.children as layoutNode[];
          const newNodeIds = new Set(children.map((n) => n.id));

          // Remove deleted nodes with fade out
          oldNodes.forEach((oldNode) => {
            if (!newNodeIds.has(oldNode.id)) {
              svg
                .select(`g:has(rect.node.id-${oldNode.id})`)
                .transition()
                .duration(300)
                .style("opacity", 0)
                .remove();
            }
          });

          // Animate existing nodes to new positions
          /**
           * PSA for this FN and interaction with the filter box
           * (or anything that will repeatedly change `componentsById`)
           *
           * Once a component gets added (in the `else`) it will be animating
           * The next time through this fn it will run the `if` because it will exist.
           *
           * All animation "end states" in the adding (`else`) need to be defined in
           * the "update" (`if`) too.
           *
           * For example, if a node gets added, and its opacity has not finished animating
           * when it runs through the update, the new transition will be applied leaving its
           * opacity "halfway through" animating. You need to have (opacity, 1) in the transition
           * definition (even if its redundant for everything already at opacity = 1)
           * */
          children.forEach(async (newNode) => {
            const existingGroup = svg.select(
              `g:has(rect.node.id-${newNode.id})`,
            );
            if (!existingGroup.empty()) {
              // Animate to new position
              existingGroup
                .transition()
                .duration(600)
                .ease(d3.easeCubicOut)
                .style("opacity", 1)
                .attr("transform", `translate(${newNode.x}, ${newNode.y})`);

              // Update selection state on existing nodes
              const rect = existingGroup.select("rect.node");
              const isSelected = selectedComponentIds.value.has(newNode.id);
              const hasFailedActions = props.componentsWithFailedActions.has(
                newNode.id,
              );

              rect.attr("class", () => {
                const classes = [`node`, `id-${newNode.id}`];
                if (isSelected) classes.push("selected");
                if (hasFailedActions) classes.push("failed-actions");
                return classes.join(" ");
              });
            } else {
              // Add new node with fade in
              const group = svg
                .append("g")
                .attr("transform", `translate(${newNode.x}, ${newNode.y})`)
                .style("opacity", 0);

              addNodeElements(group, newNode);
              group.transition().duration(400).style("opacity", 1);
            }
          });

          svg.selectAll(".edge").remove();

          renderEdges(svg, edges);

          if (selectedComponent.value) {
            panToComponent(selectedComponent.value.id);
          }

          // Selection state is already handled during animation above
        } else {
          // First render - clear and redraw immediately
          svg.selectAll("*").remove();

          children.forEach((node) => {
            const group = svg
              .append("g")
              .attr("transform", `translate(${node.x}, ${node.y})`);
            addNodeElements(group, node);
          });

          renderEdges(svg, edges);

          // Ensure selection state is applied after first render
          nextTick(() => {
            applySelectionState();
          });
        }

        // Store current layout for next animation
        previousLayout.value = layoutedData as unknown as GraphData;
      });
      // Don't show context menu when component is selected via URL parameter
      // Context menu should only show on user interaction (right-click)
    },
    100,
    { trailing: true },
  ),
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
  (e: "selectedComponents", components: Set<ComponentInList>): void;
  (e: "help"): void;
}>();

defineExpose({
  selectedComponents,
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

    &.selected.failed-actions {
      stroke: @colors-destructive-600;
      stroke-width: 3;
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
    font-size: 0.75rem;
    &.name {
      font-weight: bold;
      font-size: 0.875rem;
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
