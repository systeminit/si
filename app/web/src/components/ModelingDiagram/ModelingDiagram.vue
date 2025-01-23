/* Modeling diagram component * NOTE - uses a resize observer to react to size
changes, so this must be placed in a container that is sized explicitly has
overflow hidden */
<template>
  <div
    :style="{
      marginLeft:
        presenceStore.leftResizePanelWidth === 0
          ? '0'
          : `${LEFT_PANEL_DRAWER_WIDTH}px`, // related to left panel drawer
      marginRight:
        presenceStore.rightResizePanelWidth === 0
          ? '0'
          : `${LEFT_PANEL_DRAWER_WIDTH}px`, // related to left panel drawer, balance the diagram
    }"
    class="grow h-full relative bg-neutral-50 dark:bg-neutral-900"
  >
    <!-- This section contains the DiagramGridBackground and other elements which should render underneath all of the components/frames/cursors -->
    <div class="absolute inset-0 overflow-hidden">
      <v-stage
        v-if="customFontsLoaded && containerWidth > 0 && containerHeight > 0"
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
      <div
        v-if="fetchDiagramReqStatus.isFirstLoad"
        class="w-full h-full flex items-center bg-[rgba(0,0,0,.1)]"
      >
        <LoadingMessage message="Loading change set" />
      </div>
      <DiagramEmptyState v-else-if="viewsStore.diagramIsEmpty" />
    </div>
    <!-- This section contains the main v-stage with all of the components/frames/cursors, as well as the DiagramControls -->
    <div
      id="konva-container"
      ref="containerRef"
      :style="{ cursor }"
      class="absolute inset-0 overflow-hidden modeling-diagram"
    >
      <!-- DEBUG BAR-->
      <div
        v-if="enableDebugMode"
        class="absolute bg-black text-white flex space-x-10 z-10 opacity-50"
      >
        <div>fonts loaded? {{ customFontsLoaded }}</div>
        <div>origin = {{ gridOrigin.x }}, {{ gridOrigin.y }}</div>
        <div>
          pointer (raw) =
          {{ containerPointerPos?.x }}, {{ containerPointerPos?.y }}
        </div>
        <div>
          pointer (grid) =
          {{ gridPointerPos?.x }}, {{ gridPointerPos?.y }}
        </div>
      </div>

      <DiagramControls
        @downloadCanvasScreenshot="downloadCanvasScreenshot"
        @open:help="helpModalRef.open()"
      />

      <!-- MAIN V-STAGE -->
      <v-stage
        v-if="customFontsLoaded && containerWidth > 0 && containerHeight > 0"
        ref="stageRef"
        :config="{
          width: containerWidth,
          height: containerHeight,
          scale: { x: zoomLevel, y: zoomLevel },
          offset: { x: gridMinX, y: gridMinY },
          devicePixelRatio: 1,
        }"
        @mousedown="onMouseDown"
        @click.right="onRightClick"
      >
        <v-layer>
          <DiagramGroup
            v-for="group in groups"
            :key="group.uniqueKey"
            :connectedEdges="connectedEdgesByElementKey[group.uniqueKey]"
            :debug="enableDebugMode"
            :group="group"
            :isHovered="elementIsHovered(group)"
            :isSelected="elementIsSelected(group)"
            :qualificationStatus="
              qualificationStore.qualificationStatusForComponentId(group.def.id)
            "
            @rename="
              (f) => {
                renameOnDiagram(group, f);
              }
            "
          />
          <template v-for="node of nodes" :key="node.uniqueKey">
            <DiagramNode
              :connectedEdges="connectedEdgesByElementKey[node.uniqueKey]"
              :debug="enableDebugMode"
              :isHovered="elementIsHovered(node)"
              :isLoading="statusStore.componentIsLoading(node.def.id)"
              :isSelected="elementIsSelected(node)"
              :node="node"
              :qualificationStatus="
                qualificationStore.qualificationStatusForComponentId(
                  node.def.id,
                )
              "
              @rename="
                (f) => {
                  renameOnDiagram(node, f);
                }
              "
            />
          </template>
          <template
            v-for="view in Object.values(viewsStore.viewNodes)"
            :key="view.id"
          >
            <DiagramView
              :isHovered="elementIsHovered(view)"
              :isSelected="elementIsSelected(view)"
              :view="view.def"
            />
          </template>
          <DiagramCursor
            v-for="mouseCursor in presenceStore.diagramCursors"
            :key="mouseCursor.userId"
            :cursor="mouseCursor"
          />
          <DiagramEdge
            v-for="edge in viewsStore.edges"
            :key="edge.uniqueKey"
            :edge="edge"
            :fromPoint="getSocketLocationInfo('from', edge)?.center"
            :isHovered="elementIsHovered(edge)"
            :isSelected="elementIsSelected(edge)"
            :toPoint="getSocketLocationInfo('to', edge)?.center"
          />
          <DiagramGroupOverlay
            v-for="group in groups"
            :key="group.uniqueKey"
            :group="group"
          />

          <!-- placeholders for new inserted elements still processing -->
          <template
            v-for="(
              pendingInsert, pendingInsertId
            ) in viewsStore.pendingInsertedComponents"
            :key="pendingInsertId"
          >
            <v-rect
              :config="{
            width: 160,
            height: 80,
            cornerRadius: CORNER_RADIUS,
            x: pendingInsert.position!.x - 80,
            y: pendingInsert.position!.y + 20,
            fill: 'rgba(0,0,0,.4)',
            strokeWidth: 1,
            stroke: SELECTION_COLOR,
          }"
            />
            <DiagramIcon
              :color="getToneColorHex('info')"
              :size="60"
              :x="pendingInsert.position!.x"
              :y="pendingInsert.position!.y + 60"
              icon="loader"
            />
          </template>

          <!-- drag to select selection box -->
          <v-rect
            v-if="dragSelectActive && dragSelectStartPos && dragSelectEndPos"
            :config="{
              x: dragSelectStartPos.x,
              y: dragSelectStartPos.y,
              width: dragSelectEndPos.x - dragSelectStartPos.x,
              height: dragSelectEndPos.y - dragSelectStartPos.y,
              fill: SELECTION_BOX_INNER_COLOR,
              strokeWidth: 1,
              stroke: SELECTION_COLOR,
              listening: false,
            }"
          />

          <!-- new edge being drawn -->
          <DiagramNewEdge
            v-if="drawEdgeActive"
            :fromPoint="
              getSocketLocationInfo(undefined, undefined, drawEdgeFromSocketKey)
                ?.center
            "
            :toPoint="
              getSocketLocationInfo(undefined, undefined, drawEdgeToSocketKey)
                ?.center || gridPointerPos
            "
          />
        </v-layer>

        <!-- selection outline -->
        <v-layer>
          <v-rect
            v-for="rect in selectionRects"
            :key="`${rect.x}_${rect.y}`"
            :config="{
              x: rect.x - 9,
              y: rect.y - 9,
              width: rect.width + 18,
              height: rect.height + 18,
              cornerRadius: CORNER_RADIUS + 5,
              stroke: SELECTION_COLOR,
              strokeWidth: 3,
              listening: false,
            }"
          >
          </v-rect>
        </v-layer>
      </v-stage>

      <DiagramHelpModal ref="helpModalRef" />

      <div ref="renameInputWrapperRef" class="absolute">
        <VormInput
          ref="renameInputRef"
          v-model="renameInputValue"
          :renameZoom="zoomLevel"
          compact
          noLabel
          rename
          @blur="onRenameSubmit"
          @keydown="onRenameKeyDown"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
type DiagramContext = {
  zoomLevel: Ref<number>;
  setZoomLevel: (newZoom: number) => void;
  drawEdgeState: ComputedRef<DiagramDrawEdgeState>;
  moveElementsState: ComputedRef<MoveElementsState>;
  gridRect: ComputedRef<IRect>;
};

const DIAGRAM_CONTEXT_INJECTION_KEY: InjectionKey<DiagramContext> =
  Symbol("DIAGRAM_CONTEXT");

export function useDiagramContext() {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return inject(DIAGRAM_CONTEXT_INJECTION_KEY)!;
}
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import {
  onMounted,
  ref,
  computed,
  onBeforeUnmount,
  watch,
  InjectionKey,
  ComputedRef,
  Ref,
  provide,
  inject,
  toRaw,
} from "vue";
import { Stage as KonvaStage } from "konva/lib/Stage";
import Konva from "konva";
import * as _ from "lodash-es";
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d, IRect } from "konva/lib/types";
import tinycolor from "tinycolor2";
import {
  LoadingMessage,
  getToneColorHex,
  VormInput,
} from "@si/vue-lib/design-system";
import { connectionAnnotationFitsReference } from "@si/ts-lib/src/connection-annotations";
import { windowListenerManager } from "@si/vue-lib";
import { useRoute } from "vue-router";
import { useToast } from "vue-toastification";
import { useCustomFontsLoaded } from "@/utils/useFontLoaded";
import DiagramGroup from "@/components/ModelingDiagram/DiagramGroup.vue";
import { useComponentsStore } from "@/store/components.store";
import DiagramGroupOverlay from "@/components/ModelingDiagram/DiagramGroupOverlay.vue";
import { DiagramCursorDef, usePresenceStore } from "@/store/presence.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { useChangeSetsStore, diagramUlid } from "@/store/change_sets.store";
import { ComponentId, EdgeId } from "@/api/sdf/dal/component";
import { useViewsStore } from "@/store/views.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useStatusStore } from "@/store/status.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import { nonNullable } from "@/utils/typescriptLinter";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { DefaultMap } from "@/utils/defaultmap";
import DiagramGridBackground from "./DiagramGridBackground.vue";
import {
  DiagramDrawEdgeState,
  Direction,
  RightClickElementEvent,
  DiagramNodeData,
  DiagramGroupData,
  DiagramEdgeData,
  DiagramSocketData,
  DiagramElementData,
  DiagramElementUniqueKey,
  Size2D,
  SideAndCornerIdentifiers,
  ElementHoverMeta,
  MoveElementsState,
  DiagramViewData,
  Bounds,
  toRequiredBounds,
} from "./diagram_types";
import DiagramNode from "./DiagramNode.vue";
import DiagramCursor from "./DiagramCursor.vue";
import DiagramEdge from "./DiagramEdge.vue";
import {
  DRAG_DISTANCE_THRESHOLD,
  DRAG_EDGE_TRIGGER_SCROLL_WIDTH,
  SOCKET_SIZE,
  CORNER_RADIUS,
  SELECTION_COLOR,
  MAX_ZOOM,
  MIN_ZOOM,
  GROUP_INTERNAL_PADDING,
  GROUP_BOTTOM_INTERNAL_PADDING,
  GROUP_INNER_Y_BOUNDARY_OFFSET,
  MIN_NODE_DIMENSION,
  GROUP_HEADER_BOTTOM_MARGIN,
  NODE_TITLE_HEADER_MARGIN_RIGHT,
  GROUP_HEADER_ICON_SIZE,
  NODE_HEADER_HEIGHT,
} from "./diagram_constants";
import {
  vectorDistance,
  vectorAdd,
  checkRectanglesOverlap,
  rectContainsAnother,
  vectorBetween,
  pointAlongLinePct,
  getRectCenter,
  getAdjustmentRectToContainAnother,
  pointsToRect,
  rectContainsPoint,
  vectorSubtract,
} from "./utils/math";
import DiagramNewEdge from "./DiagramNewEdge.vue";
import { convertArrowKeyToDirection } from "./utils/keyboard";
import DiagramControls from "./DiagramControls.vue";
import DiagramHelpModal from "./DiagramHelpModal.vue";
import DiagramIcon from "./DiagramIcon.vue";
import DiagramEmptyState from "./DiagramEmptyState.vue";
import DiagramView from "./DiagramView.vue";

const LEFT_PANEL_DRAWER_WIDTH = 230;

// SET THIS BOOLEAN TO TRUE TO ENABLE DEBUG MODE!
// VERY HELPFUL FOR DEBUGGING ISSUES ON THE DIAGRAM!
const enableDebugMode = false;

const route = useRoute();
const toast = useToast();

const changeSetsStore = useChangeSetsStore();
const realtimeStore = useRealtimeStore();
const qualificationStore = useQualificationsStore();

// scroll pan multiplied by this and zoom level when panning
const ZOOM_PAN_FACTOR = 0.5;

const props = defineProps<{
  cursors?: DiagramCursorDef[];
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  (e: "right-click-element", elRightClickInfo: RightClickElementEvent): void;
  (e: "close-right-click-menu"): void;
}>();

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();
const statusStore = useStatusStore();
const featureFlagsStore = useFeatureFlagsStore();
const modelingEventBus = componentsStore.eventBus;

const fetchDiagramReqStatus = viewsStore.getRequestStatus("FETCH_VIEW");

const customFontsLoaded = useCustomFontsLoaded();

let kStage: KonvaStage;
const stageRef = ref();
const containerRef = ref<HTMLDivElement>();

// we track the container dimensions and position locally here using a resize observer
// so if the outside world wants to resize the diagram, it should just resize whatever container it lives in
const containerWidth = ref(0);
const containerHeight = ref(0);
const containerViewportX = ref(0);
const containerViewportY = ref(0);

// we'll manage the canvas origin/panning locally, since it's never exposed to the user
// and we're choosing to keep our origin (defaulting to 0,0) at the center of the diagram to keep things (hopefully) simpler
const gridOrigin = ref<Vector2d>({ x: 0, y: 0 });

// zoom level (1 = 100%)
// I opted to track this internally rather than use v-model so the parent component isn't _forced_ to care about it
// but there will often probably be some external controls, which can be done using exposed setZoom and update:zoom event
const zoomLevel = ref(1);

function setZoomLevel(newZoomLevel: number) {
  emit("close-right-click-menu");

  if (newZoomLevel < MIN_ZOOM) zoomLevel.value = MIN_ZOOM;
  else if (newZoomLevel > MAX_ZOOM) zoomLevel.value = MAX_ZOOM;
  else zoomLevel.value = newZoomLevel;

  if (zoomLevel.value === 1) {
    window.localStorage.removeItem("si-diagram-zoom");
  } else {
    window.localStorage.setItem("si-diagram-zoom", `${zoomLevel.value}`);
  }
}

// dimensions of our 2d grid space, all coordinates of things in the diagram are relative to this
const gridWidth = computed(() => containerWidth.value / zoomLevel.value);
const gridHeight = computed(() => containerHeight.value / zoomLevel.value);
// min/max values of the visible region of the diagram
const gridMinX = computed(() => gridOrigin.value.x - gridWidth.value / 2);
const gridMaxX = computed(() => gridOrigin.value.x + gridWidth.value / 2);
const gridMinY = computed(() => gridOrigin.value.y - gridHeight.value / 2);
const gridMaxY = computed(() => gridOrigin.value.y + gridHeight.value / 2);

const gridRect = computed<IRect>(() => ({
  x: gridMinX.value,
  y: gridMinY.value,
  width: gridWidth.value,
  height: gridHeight.value,
}));

function convertContainerCoordsToGridCoords(v: Vector2d): Vector2d {
  return {
    x: gridMinX.value + v.x / zoomLevel.value,
    y: gridMinY.value + v.y / zoomLevel.value,
  };
}

function convertGridCoordsToContainerCoords(v: Vector2d): Vector2d {
  return {
    x: (v.x - gridMinX.value) * zoomLevel.value,
    y: (v.y - gridMinY.value) * zoomLevel.value,
  };
}

/** pointer position in frame of reference of container */
const containerPointerPos = ref<Vector2d>();
/** pointer position in frame of reference of grid  */
const gridPointerPos = computed(() => {
  if (!containerPointerPos.value) return undefined;
  if (route.name === "workspace-lab") return undefined;
  const converted = convertContainerCoordsToGridCoords(
    containerPointerPos.value,
  );
  converted.x = Math.round(converted.x);
  converted.y = Math.round(converted.y);
  return converted;
});
watch(gridPointerPos, (pos) => {
  if (!pos) return;
  sendUpdatedPointerPos(pointerIsWithinGrid.value ? pos : undefined);
});

const presenceStore = usePresenceStore();

function sendUpdatedPointerPos(pos?: Vector2d) {
  presenceStore.updateCursor(pos ?? null);
}

const MIN_LEFT_BAR_WIDTH = 430;
const pointerIsWithinGrid = computed(() => {
  if (!gridPointerPos.value) return false;
  const modifier = presenceStore.leftDrawerOpen ? MIN_LEFT_BAR_WIDTH : 0;
  const { x, y } = gridPointerPos.value;
  if (x < gridMinX.value + modifier || x > gridMaxX.value) return false;
  if (y < gridMinY.value || y > gridMaxY.value) return false;
  return true;
});

function onMouseWheel(e: KonvaEventObject<WheelEvent>) {
  // TODO check if target is the stage?
  e.evt.preventDefault();

  emit("close-right-click-menu");

  // is it a mouse wheel or a trackpad pinch to zoom?
  const isTrackpadPinch = !_.isInteger(e.evt.deltaY);
  // if CMD key, treat wheel as zoom, otherwise pan
  if (e.evt.metaKey || (e.evt.ctrlKey && isTrackpadPinch)) {
    // need to move origin to zoom centered on pointer position
    if (containerPointerPos.value && gridPointerPos.value) {
      // this a little confusing, but we're recreating the same calculations as above, but but at the new zoom level
      // so we know where the pointer _would_ move and then offset the pointer position stays constant
      zoomAtPoint(e.evt.deltaY, containerPointerPos.value, isTrackpadPinch);
    }
  } else {
    // pan
    const panFactor = ZOOM_PAN_FACTOR / zoomLevel.value;
    gridOrigin.value = {
      x: gridOrigin.value.x + e.evt.deltaX * panFactor,
      y: gridOrigin.value.y + e.evt.deltaY * panFactor,
    };
  }
  fixRenameInputPosition();
}

function zoomAtPoint(delta: number, zoomPos: Vector2d, isPinchToZoom = false) {
  // e.evt.metaKey
  // zoom
  if (!gridPointerPos.value) return;

  const panSpeed = 0.001 * (isPinchToZoom ? 20 : 1) * zoomLevel.value;

  let newZoomLevel = zoomLevel.value - delta * panSpeed;
  if (newZoomLevel < MIN_ZOOM) newZoomLevel = MIN_ZOOM;
  if (newZoomLevel > MAX_ZOOM) newZoomLevel = MAX_ZOOM;

  const newGridWidth = containerWidth.value / newZoomLevel;
  const newMinX = gridOrigin.value.x - newGridWidth / 2;
  const newGridHeight = containerHeight.value / newZoomLevel;
  const newMinY = gridOrigin.value.y - newGridHeight / 2;
  const pointerXAtNewZoom = newMinX + zoomPos.x / newZoomLevel;
  const pointerYAtNewZoom = newMinY + zoomPos.y / newZoomLevel;

  gridOrigin.value = {
    x: gridOrigin.value.x - (pointerXAtNewZoom - gridPointerPos.value.x),
    y: gridOrigin.value.y - (pointerYAtNewZoom - gridPointerPos.value.y),
  };

  setZoomLevel(newZoomLevel);
}

// not sure why but TS couldnt quite find the ResizeObserverCallback type...
type ResizeObserverCallback = ConstructorParameters<typeof ResizeObserver>[0];
const onResize: ResizeObserverCallback = (entries) => {
  entries.forEach((entry) => {
    if (!containerRef.value || entry.target !== containerRef.value) return;

    // using the resize observer lets us listen for resizes
    // and we'll assume location changes also happen with resizes for now

    // but resize observer won't help us get the element's position within the window
    // so we still call getBoundingClientRect

    // bounding rect helps us get the location of the container in the window
    const boundingRect = containerRef.value.getBoundingClientRect();
    containerViewportX.value = boundingRect.x;
    containerViewportY.value = boundingRect.y;

    containerWidth.value = boundingRect.width; // also available as entry.contentRect.width, entry.contentRect.height;
    containerHeight.value = boundingRect.height;
  });
};
// debounce the resize handler... might want to play with this delay value
const debouncedOnResize = _.debounce(onResize, 50);
const resizeObserver = new ResizeObserver(debouncedOnResize);

// this is all a little ugly, but basically we are waiting until custom fonts are loaded to initialize and display the canvas
// or otherwise spacing gets messed up and we'd have to tell everything to rerender/recalculate when the fonts did get loaded
const isMounted = ref(false);
onMounted(() => {
  resizeObserver.observe(containerRef.value!);
  isMounted.value = true;
  const lastZoomValue = window.localStorage.getItem("si-diagram-zoom");
  if (lastZoomValue) {
    zoomLevel.value = Number(lastZoomValue);
  }
});

const CLIPBOARD_LOCALSTORAGE_KEY = computed(
  () => `clipboard-si-${changeSetsStore.selectedChangeSetId}`,
);

watch([customFontsLoaded, () => isMounted.value, () => stageRef.value], () => {
  if (!isMounted.value || !customFontsLoaded.value || !stageRef.value) return;
  onMountedAndReady();
});

function onMountedAndReady() {
  kStage = stageRef.value.getNode();
  kStage.on("wheel", onMouseWheel);
  // attach to window so we have coords even when mouse is outside bounds or on other elements
  // NOTE - mousedown is attached on the konva stage component above, since we only care about starting clicks within the diagram
  windowListenerManager.addEventListener("mousemove", onMouseMove);
  windowListenerManager.addEventListener("mouseup", onMouseUp);
  windowListenerManager.addEventListener("keydown", onKeyDown);
  windowListenerManager.addEventListener("keyup", onKeyUp);

  // window.addEventListener("pointerdown", onPointerDown);
  // window.addEventListener("pointermove", onPointerMove);
  // window.addEventListener("pointerup", onPointerUp);
  // window.addEventListener("pointercancel", onPointerUp);
  // window.addEventListener("pointerout", onPointerUp);
  // window.addEventListener("pointerleave", onPointerUp);
}

let executionKey: string | undefined;
watch(
  () => changeSetsStore.selectedChangeSetId,
  () => {
    if (executionKey) {
      // this doesnt seem to fire (see below)
      realtimeStore.unsubscribe(executionKey);
    }

    executionKey = new Date().toString() + _.random();
    realtimeStore.subscribe(
      executionKey,
      `changeset/${changeSetsStore.selectedChangeSetId}`,
      [
        {
          eventType: "SetComponentPosition",
          callback: ({ changeSetId, clientUlid, positions }) => {
            if (changeSetId !== changeSetsStore.selectedChangeSetId) return;
            if (clientUlid === diagramUlid) return;
            // TODO: make sure to update the correct view based on ID

            for (const geo of positions) {
              const component =
                componentsStore.allComponentsById[geo.componentId];
              if (component) {
                let viewComponent;
                if (component.def.isGroup) {
                  viewComponent = viewsStore.groups[geo.componentId];
                  if (viewComponent && geo.height && geo.width) {
                    viewComponent.height = geo.height;
                    viewComponent.width = geo.width;
                  }
                } else {
                  viewComponent = viewsStore.components[geo.componentId];
                }
                if (viewComponent) {
                  viewComponent.x = geo.x;
                  viewComponent.y = geo.y;
                }
              }
            }
          },
        },
      ],
    );
  },
  { immediate: true },
);

function downloadCanvasScreenshot() {
  const stage = stageRef.value.getNode() as Konva.Stage;
  if (!stage || typeof stage.getLayers !== "function") return;

  // Find the bounding box of all shapes on the stage
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  stage.find("Shape, Text, Image").forEach((node) => {
    const box = node.getClientRect();
    minX = Math.min(minX, box.x);
    minY = Math.min(minY, box.y);
    maxX = Math.max(maxX, box.x + box.width);
    maxY = Math.max(maxY, box.y + box.height);
  });

  // Add a small padding around the components on the graph (adjust as needed)
  const padding = 10;
  minX -= padding;
  minY -= padding;
  maxX += padding;
  maxY += padding;

  const width = maxX - minX;
  const height = maxY - minY;

  // Create a temporary layer for the background
  const tempLayer = new Konva.Layer();
  stage.add(tempLayer);

  const bgRect = new Konva.Rect({
    x: minX,
    y: minY,
    width,
    height,
    fill: "black",
  });
  tempLayer.add(bgRect);
  tempLayer.moveToBottom();

  // Capture the content
  const dataURL = stage.toDataURL({
    x: minX,
    y: minY,
    width,
    height,
    pixelRatio: 10, // Increase for higher resolution
  });

  // Remove the temporary layer
  tempLayer.destroy();

  // Generate filename with current date
  const currentDate = new Date();
  const dateString = currentDate.toISOString().split("T")[0];
  const fileName = `canvas-screenshot-${dateString}.png`;

  // Create a link element
  const link = document.createElement("a");
  link.download = fileName;
  link.href = dataURL;

  // Trigger download
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}

onBeforeUnmount(() => {
  // this fires when you change the change set from the drop down
  // which feels unexpected that this component is destroyed and recreated?
  if (executionKey) realtimeStore.unsubscribe(executionKey);

  kStage?.off("wheel", onMouseWheel);
  windowListenerManager.removeEventListener("mousemove", onMouseMove);
  windowListenerManager.removeEventListener("mouseup", onMouseUp);
  windowListenerManager.removeEventListener("keydown", onKeyDown);
  windowListenerManager.removeEventListener("keyup", onKeyUp);

  // window.removeEventListener("pointerdown", onPointerDown);
  // window.removeEventListener("pointermove", onPointerMove);
  // window.removeEventListener("pointerup", onPointerUp);
  // window.removeEventListener("pointercancel", onPointerUp);
  // window.removeEventListener("pointerout", onPointerUp);
  // window.removeEventListener("pointerleave", onPointerUp);
  resizeObserver.unobserve(containerRef.value!);
});

// global keyboard and mouse handlers, which reroute to the correct functionality

const spaceKeyIsDown = ref(false);
const shiftKeyIsDown = ref(false);

async function onKeyDown(e: KeyboardEvent) {
  // TODO: check is cursor is within graph bounds
  // TODO: check if something else (like an input) is focused and bail

  // if focused on an input (or anything) dont do anything, let normal behaviour proceed
  // TODO: this should be more sophisticated
  if (document?.activeElement?.tagName !== "BODY") return;

  // handle opening the help modal
  if (e.key === "?" || e.key === "/") helpModalRef.value?.open();

  // handle zoom hotkeys
  if (e.key === "=" || e.key === "+") {
    setZoomLevel(zoomLevel.value + 0.1);
  }
  if (e.key === "-" || e.key === "_") {
    setZoomLevel(zoomLevel.value - 0.1);
  }
  // CMD + 0 - reset zoom level to 100%
  if (e.key === "0" && e.metaKey) {
    setZoomLevel(1);
  }

  if (
    (e.metaKey || e.ctrlKey) &&
    e.key === "c" &&
    viewsStore.selectedComponentIds.length
  ) {
    const component = viewsStore.selectedComponents
      .filter(
        (c): c is DiagramNodeData | DiagramGroupData =>
          !(c instanceof DiagramViewData),
      )
      .pop();
    const containsUpgradeable = viewsStore.selectedComponents.some(
      (c) => "canBeUpgraded" in c.def && c.def.canBeUpgraded,
    );
    if (containsUpgradeable) {
      toast("Components that can be upgraded cannot be copied");
    } else if (component) {
      // TODO: how to get copyingFrom
      window.localStorage.setItem(
        CLIPBOARD_LOCALSTORAGE_KEY.value,
        JSON.stringify({
          componentIds: viewsStore.selectedComponentIds,
          copyingFrom: component.def.isGroup
            ? { ...viewsStore.groups[component.def.id] }
            : { ...viewsStore.components[component.def.id] },
        }),
      );
    }
  } else if ((e.ctrlKey || e.metaKey) && e.key === "v") {
    const json = window.localStorage.getItem(CLIPBOARD_LOCALSTORAGE_KEY.value);
    if (json !== null && json !== "null") {
      try {
        const { componentIds, copyingFrom } = JSON.parse(json);
        viewsStore.selectedComponentIds = componentIds;
        componentsStore.copyingFrom = copyingFrom;
        triggerPasteElements();
      } catch {
        //
      }
    }
  }

  // handle arrow keys - nudge and alignment
  if (!props.readOnly && e.key.startsWith("Arrow")) {
    const direction = convertArrowKeyToDirection(e.key);
    if (e.metaKey) alignSelection(direction);
    else nudgeSelection(direction, e.shiftKey);
    // CMD left/right controls browser back/fwd, so we need to prevent default
    e.preventDefault();
  }

  // handle recording modifier keys globally, which can be useful elsewhere
  if (e.key === " ") spaceKeyIsDown.value = true;
  if (e.key === "Shift") shiftKeyIsDown.value = true;

  // TODO: probably want to consider repeat keydown events for using arrows to move stuff
  if (e.repeat) return; // don't process repeat events (key being held down fires multiple times)

  // TODO: escape will probably have more complex behaviour
  if (e.key === "Escape") {
    clearSelection();
    renameHide();
    if (insertElementActive.value) componentsStore.cancelInsert();
    componentsStore.copyingFrom = null;
    if (dragSelectActive.value) endDragSelect(false);
  }
  if (!props.readOnly && (e.key === "Delete" || e.key === "Backspace")) {
    modelingEventBus.emit("deleteSelection");
  }
  if (!props.readOnly && e.key === "e" && e.metaKey) {
    modelingEventBus.emit("eraseSelection");
  }
  if (e.key === "a" && e.metaKey) {
    // TODO(Wendy) - select all!
  }
  if (
    !props.readOnly &&
    e.key === "r" &&
    viewsStore.selectedComponent?.def &&
    "hasResource" in viewsStore.selectedComponent.def &&
    viewsStore.selectedComponent?.def.hasResource &&
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId
  ) {
    componentsStore.REFRESH_RESOURCE_INFO(viewsStore.selectedComponent.def.id);
  }
  if (
    !props.readOnly &&
    e.key === "n" &&
    viewsStore.selectedComponentId &&
    viewsStore.selectedComponent &&
    !(viewsStore.selectedComponent instanceof DiagramViewData)
  ) {
    e.preventDefault();
    renameOnDiagramByComponentId(viewsStore.selectedComponentId);
  }
  if (
    !props.readOnly &&
    featureFlagsStore.TEMPLATE_MGMT_FUNC_GENERATION &&
    e.key === "t" &&
    viewsStore.restorableSelectedComponents.length === 0 &&
    viewsStore.selectedComponents.length > 0 &&
    !viewsStore.selectedComponents.some((c) => c instanceof DiagramViewData)
  ) {
    e.preventDefault();
    modelingEventBus.emit("templateFromSelection");
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.key === " ") spaceKeyIsDown.value = false;
  if (e.key === "Shift") {
    shiftKeyIsDown.value = false;
    // shift constrains to vertical or horizontal drag, so letting go snaps things back to the mouse position
    if (dragElementsActive.value) onDragElementsMove();
  }
}

const mouseIsDown = ref(false);
const dragThresholdBroken = ref(false);
const lastMouseDownEvent = ref<MouseEvent>();
const prevDragTotal = ref<Vector2d>({ x: 0, y: 0 });
const lastMouseDownContainerPointerPos = ref<Vector2d>();
const lastMouseDownElementKey = ref<DiagramElementUniqueKey>();
const lastMouseDownHoverMeta = ref<ElementHoverMeta | null>(null);
const lastMouseDownElement = computed(() =>
  lastMouseDownElementKey.value
    ? allElementsByKey.value[lastMouseDownElementKey.value]
    : undefined,
);

function onMouseDown(ke: KonvaEventObject<MouseEvent>) {
  // not sure why, but this is being called twice, once with the konva event, and once with the bare event
  // so we ignore the bare event
  if (!ke.evt) return;
  const e = ke.evt;

  // we dont care about right click here
  if (e.button === 2) return;

  // if the user is holding the control key their mouse click will be processed as a right click
  if (e.ctrlKey) return;

  mouseIsDown.value = true;
  dragThresholdBroken.value = false;
  lastMouseDownContainerPointerPos.value = containerPointerPos.value;
  // store the mouse event, as we may want to know modifier keys that were held on the original click
  lastMouseDownEvent.value = e;
  // track the originally clicked element, as the mouse may move off of it while beginning the drag
  lastMouseDownElementKey.value = hoveredElementKey.value;
  // track the hover meta at the time of mousedown (ex: resize, socket, etc)
  lastMouseDownHoverMeta.value = hoveredElementMeta.value;

  // for drag to pan, we start dragging right away since the user has enabled it by holding the space bar
  // for all other interactions, we watch to see if the user drags past some small threshold to begin the "drag"
  // in order to ignore clicks with a tiny bit of movement
  if (dragToPanArmed.value || e.button === 1) beginDragToPan();
  else if (insertElementActive.value) triggerInsertElement();
  else if (outlinerAddActive.value) triggerAddToView();
  else if (viewAddActive.value) triggerAddViewToView();
  else if (pasteElementsActive.value) triggerPasteElements();
  else handleMouseDownSelection();
}

function onMouseUp(e: MouseEvent) {
  // we dont care about right click
  if (e.button === 2) return;
  const target = e.target as HTMLElement;
  mouseIsDown.value = false;
  if (dragToPanActive.value) endDragToPan();
  else if (dragElementsActive.value) endDragElements();
  else if (dragSelectActive.value) endDragSelect();
  else if (drawEdgeActive.value) endDrawEdge();
  else if (resizeElementActive.value) endResizeElement();
  // we'll handle insert on mouseup too in case the user dragged the element from the asset palette and then let go in the canvas
  // TODO: probably change this - its a bit hacky...
  else if (insertElementActive.value && pointerIsWithinGrid.value)
    triggerInsertElement();
  else if (outlinerAddActive.value && pointerIsWithinGrid.value)
    triggerAddToView();
  else if (pasteElementsActive.value && pointerIsWithinGrid.value)
    triggerPasteElements();
  else if (target.nodeName === "CANVAS")
    // we're seeing mouse up firing when clicking on side rails
    handleMouseUpSelection();
}

function onMouseMove(e: MouseEvent) {
  // update pointer location relative to container, which is used throughout
  containerPointerPos.value = {
    x: e.clientX - containerViewportX.value,
    y: e.clientY - containerViewportY.value,
  };

  // some bugs where letting go of shift key isn't caught, so we'll add this here to help
  shiftKeyIsDown.value = e.shiftKey;

  if (dragToPanActive.value) onDragToPanMove();
  else if (dragElementsActive.value) onDragElementsMove();
  else if (dragSelectActive.value) onDragSelectMove();
  else if (drawEdgeActive.value) onDrawEdgeMove();
  else if (resizeElementActive.value) onResizeMove();
  else {
    if (
      mouseIsDown.value &&
      !dragThresholdBroken.value &&
      !dragToPanArmed.value &&
      !dragToPanActive.value
    ) {
      checkIfDragStarted(e);
    }
  }
}

function onRightClick(ke: KonvaEventObject<MouseEvent>) {
  const e = ke.evt;
  e.preventDefault(); // do not show browser right click menu
  if (!hoveredElement.value) return;

  if (!currentSelectionElements.value.includes(hoveredElement.value)) {
    if (shiftKeyIsDown.value && hoveredElementKey.value) {
      toggleSelectedByKey(hoveredElementKey.value);
    } else setSelectionByKey(hoveredElement.value.uniqueKey);
  }

  emit("right-click-element", {
    element: hoveredElement.value,
    e,
  });
}

function checkIfDragStarted(_e: MouseEvent) {
  if (!lastMouseDownContainerPointerPos.value || !containerPointerPos.value)
    return;
  const dragDistance = vectorDistance(
    lastMouseDownContainerPointerPos.value,
    containerPointerPos.value,
  );
  // exit early if we haven't hit the drag threshold yet
  if (dragDistance < DRAG_DISTANCE_THRESHOLD) return;

  // now user has broken the drag threshold to start dragging
  dragThresholdBroken.value = true;

  // determine what kind of drag this is and trigger it
  if (!lastMouseDownElement.value || _e.altKey) {
    // begin drag to multi-select - NOTE - alt/option forces drag to select
    beginDragSelect();
  } else if (props.readOnly) {
    // TODO: add controls for each of these modes...
    return;
  } else if ("componentType" in lastMouseDownElement.value.def) {
    if (lastMouseDownHoverMeta.value?.type === "resize") {
      beginResizeElement();
    } else if (
      "changeStatus" in lastMouseDownElement.value.def &&
      lastMouseDownElement.value.def.changeStatus !== "deleted" &&
      lastMouseDownHoverMeta.value?.type === "socket"
    ) {
      // begin drawing edge
      beginDrawEdge(lastMouseDownHoverMeta.value.socket);
    } else {
      // begin moving selected nodes (and eventually other movable things like groups / annotations, etc...)
      beginDragElements();
    }
  } else {
    throw new Error("Dragging failed instance check");
  }
}

// Pointer events (for touch screens)
// const pointerEventCache = {} as Record<number, PointerEvent>;
// let previousPointerDiff: number | undefined;

// function onPointerDown(e: PointerEvent) {
//   pointerEventCache[e.pointerId] = e;
// }

// function onPointerMove(e: PointerEvent) {
//   const events = _.values(pointerEventCache);
//   if (events.length === 2 && events[0] && events[1]) {
//     // time to zoom!
//     const point1 = { x: events[0].clientX, y: events[0].clientY };
//     const point2 = { x: events[1].clientX, y: events[1].clientY };
//     const zoomCenter = pointAlongLinePct(point1, point2, 0.5);
//     const newPointerDiff = vectorDistance(point1, point2);

//     if (!previousPointerDiff) {
//       previousPointerDiff = newPointerDiff;
//     } else {
//       const delta = newPointerDiff - previousPointerDiff;
//       zoomAtPoint(delta, zoomCenter);
//     }
//     previousPointerDiff = newPointerDiff;
//   }
// }

// function onPointerUp(e: PointerEvent) {
//   delete pointerEventCache[e.pointerId];

//   if (_.values(pointerEventCache).length < 2) {
//     previousPointerDiff = undefined;
//   }
// }

// Mode and cursor
const cursor = computed(() => {
  if (dragToPanActive.value) return "grabbing";
  if (dragToPanArmed.value) return "grab";
  if (dragSelectActive.value) return "crosshair";

  if (drawEdgeActive.value) return "cell";
  if (dragElementsActive.value) return "move";
  if (insertElementActive.value) return "copy"; // not sure about this...
  if (outlinerAddActive.value) return "copy";
  if (pasteElementsActive.value) return "copy";
  if (
    resizeElementActive.value ||
    hoveredElementMeta.value?.type === "resize"
  ) {
    let dir = resizeElementActive.value && resizeElementDirection.value;
    if (!dir && hoveredElementMeta.value?.type === "resize") {
      dir = hoveredElementMeta.value?.direction;
    }
    switch (dir) {
      case "left":
      case "right":
        return "ew-resize";
      case "bottom":
      case "top":
        return "ns-resize";
      case "bottom-left":
      case "top-right":
        return "nesw-resize";
      case "bottom-right":
      case "top-left":
        return "nwse-resize";
      default:
        return "auto";
    }
  }
  if (
    !props.readOnly &&
    hoveredElementMeta.value?.type === "socket" &&
    hoveredElement.value?.def.changeStatus !== "deleted"
  ) {
    return "cell";
  }

  if (hoveredElement.value) {
    if (hoveredElementMeta.value?.type === "rename") {
      return "text";
    }
    return "pointer";
  }
  return "auto";
});

// HOVERING LOGIC + BEHAVIOUR //////////////////////////////////////////
const hoveredElementKey = computed(() => {
  // dont recompute this while we're dragging
  if (dragElementsActive.value) return undefined;

  if (viewsStore.hoveredComponentId) {
    return getDiagramElementKeyForComponentId(viewsStore.hoveredComponentId);
  } else if (viewsStore.hoveredEdgeId) {
    return DiagramEdgeData.generateUniqueKey(viewsStore.hoveredEdgeId);
  }
  return undefined;
});

const hoveredElement = computed(() => {
  // dont recompute this while we're dragging
  if (!hoveredElementKey.value) return undefined;

  let elm = hoveredElementKey.value
    ? (allElementsByKey.value[hoveredElementKey.value] as
        | DiagramEdgeData
        | DiagramGroupData
        | DiagramNodeData)
    : undefined;
  if (!elm) {
    // putting this last, using a find
    const id = hoveredElementKey.value?.substring(2);
    elm = viewsStore.edges.find((edge) => edge.def.id === id);
  }
  return elm;
});

// same event and handler is used for both hovering nodes and sockets
// NOTE - we'll receive 2 events when hovering sockets, one for the node and one for the socket

// more detailed info about what inside an element is being hovered (like resize direction, socket, etc)
const hoveredElementMeta = computed(() => viewsStore.hoveredComponentMeta);

const disableHoverEvents = computed(() => {
  if (dragToPanArmed.value || dragToPanActive.value) return true;
  if (dragElementsActive.value) return true;
  if (dragSelectActive.value) return true;
  if (drawEdgeActive.value) return true;
  if (resizeElementActive.value) return true;
  // TODO: other states will  disable hovers, like drawing an edge, or dragging a selection box
  return false;
});

// DRAG TO PAN (pan using click and drag while space bar is held down) ////////////////////////////////////
const dragToPanArmed = computed(() => spaceKeyIsDown.value); // hold space to enable
const dragToPanActive = ref(false); // then click to start dragging

const beginDragOrigin = ref<Vector2d | null>(null);

function beginDragToPan() {
  if (!containerPointerPos.value) return;
  dragToPanActive.value = true;
  beginDragOrigin.value = gridOrigin.value;
}

function onDragToPanMove() {
  if (!beginDragOrigin.value || !lastMouseDownContainerPointerPos.value) return;
  if (!containerPointerPos.value) return;

  // we are using the container position rather than grid position here because
  // the grid position is relative to the origin, which is being changed here
  gridOrigin.value = {
    x:
      beginDragOrigin.value.x +
      (lastMouseDownContainerPointerPos.value.x - containerPointerPos.value.x) /
        zoomLevel.value,
    y:
      beginDragOrigin.value.y +
      (lastMouseDownContainerPointerPos.value.y - containerPointerPos.value.y) /
        zoomLevel.value,
  };
}

function endDragToPan() {
  dragToPanActive.value = false;
}

// AUTO PAN (pan using click and drag while space bar is held down) ////////////////////////////////////
watch(
  () => componentsStore.panTargetComponentId,
  () => {
    if (!componentsStore.panTargetComponentId) return;

    const panToComponent =
      componentsStore.allComponentsById[componentsStore.panTargetComponentId];
    if (!panToComponent) return;

    recenterOnElement(panToComponent);
    componentsStore.panTargetComponentId = null;
  },
);

// TODO: handle multiple components?
function panToComponent(payload: {
  component: DiagramGroupData | DiagramGroupData;
  center?: boolean;
}) {
  const nodeRect =
    payload.component instanceof DiagramNodeData
      ? viewsStore.components[payload.component.def.id]
      : viewsStore.groups[payload.component.def.id];
  if (!nodeRect) return;

  if (payload.center) {
    // TODO: if element doesnt fit on screen, need to zoom out
    gridOrigin.value = getRectCenter(nodeRect);
  } else if (!rectContainsAnother(gridRect.value, nodeRect)) {
    // current behaviour will adjust the grid so the component is just moved onscreen plus some small buffer
    // we could also decide to recenter if its totally off screen, and just move slightly otherwise?
    // also could explore animating that change so you can see where it was?
    const adjustmentVector = getAdjustmentRectToContainAnother(
      gridRect.value,
      nodeRect,
    );
    gridOrigin.value.x += adjustmentVector.x;
    gridOrigin.value.y += adjustmentVector.y;
  }
}

// ELEMENT SELECTION /////////////////////////////////////////////////////////////////////////////////
const currentSelectionKeys = computed(() => {
  if (viewsStore.selectedEdgeId) {
    return _.compact([
      getDiagramElementKeyForEdgeId(viewsStore.selectedEdgeId),
    ]);
  } else {
    return _.compact(
      _.map(viewsStore.selectedComponentIds, (componentId) => {
        let component:
          | DiagramNodeData
          | DiagramGroupData
          | DiagramViewData
          | undefined = componentsStore.allComponentsById[componentId];
        if (!component) component = viewsStore.viewNodes[componentId];
        return component?.uniqueKey;
      }),
    );
  }
});
const currentSelectionElements = computed(
  () =>
    _.map(
      currentSelectionKeys.value,
      (key) => allElementsByKey.value?.[key],
    ).filter((element) => !!element) as DiagramElementData[],
);

function setSelectionByKey(
  toSelect?: DiagramElementUniqueKey | DiagramElementUniqueKey[],
) {
  if (!toSelect || !toSelect.length) {
    viewsStore.setSelectedComponentId(null);
    return;
  }

  const els = _.compact(_.map(_.castArray(toSelect), getElementByKey));

  // TODO: unsure if this edge check works
  if (els.length === 1 && els[0] instanceof DiagramEdgeData) {
    viewsStore.setSelectedEdgeId(els[0].def.id);
  } else {
    const ids: string[] = [];
    els.forEach((e) => {
      if ("componentId" in e.def) ids.push(e.def.componentId);
      else if ("componentType" in e.def) ids.push(e.def.id); // view
    });
    viewsStore.setSelectedComponentId(ids);
  }
}

// toggles selected items in the selection (used when shift clicking)
function toggleSelectedByKey(
  toToggle: DiagramElementUniqueKey | DiagramElementUniqueKey[],
) {
  const els = _.compact(_.map(_.castArray(toToggle), getElementByKey));
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const elIds: string[] = [];
  els.forEach((el) => {
    if ("componentId" in el.def) elIds.push(el.def.componentId);
    else if ("componentType" in el.def) elIds.push(el.def.id); // view
  });
  // second true enables "toggle" mode
  viewsStore.setSelectedComponentId(elIds, { toggle: true });
}

function clearSelection() {
  viewsStore.setSelectedComponentId(null);
}

function elementIsHovered(el: DiagramElementData) {
  return !disableHoverEvents.value && hoveredElementKey.value === el.uniqueKey;
}

function elementIsSelected(el: DiagramElementData) {
  if (dragSelectActive.value) {
    return dragSelectPreviewKeys.value.includes(el.uniqueKey);
  } else {
    return currentSelectionKeys.value.includes(el.uniqueKey);
  }
}

const handleSelectionOnMouseUp = ref(false);

function handleMouseDownSelection() {
  handleSelectionOnMouseUp.value = false;

  // handle clicking nothing / background grid
  if (!hoveredElementKey.value) {
    // we clear selection on mousedown unless shift is held
    // in which case it could be beginning of drag to select, so we handle on mouseup
    if (shiftKeyIsDown.value) handleSelectionOnMouseUp.value = true;
    else clearSelection();
    return;
  }

  // nodes can be multi-selected, so we have some extra behaviour
  // TODO: other elements may also share this behaviour
  if (hoveredElement.value && hoveredElement.value.def) {
    // when clicking on an element that is NOT currently selected, we act right away
    // but if the element IS selected, this could be beginning of dragging
    // so we handle selection on mouseup if the user never fully started to drag
    if (!currentSelectionKeys.value.includes(hoveredElementKey.value)) {
      if (shiftKeyIsDown.value) toggleSelectedByKey(hoveredElementKey.value);
      else setSelectionByKey(hoveredElementKey.value);
    } else {
      handleSelectionOnMouseUp.value = true;
    }
  } else {
    setSelectionByKey(hoveredElementKey.value);
  }
}

// handles selection on mouseup for scenarios where the user _might_ have started dragging but did not
// see handleMouseDownSelection() for when those take place
// NOTE - this only fires if the user never breaks the drag distance threshold
function handleMouseUpSelection() {
  if (!handleSelectionOnMouseUp.value || dragThresholdBroken.value) return;
  const clickedEl = lastMouseDownElement.value;

  if (!clickedEl) clearSelection();
  else if (lastMouseDownEvent.value?.shiftKey)
    toggleSelectedByKey(clickedEl.uniqueKey);
  else setSelectionByKey(clickedEl.uniqueKey);
}

// DRAG SELECT BOX //////////////////////////////////////////////////////
const dragSelectActive = ref(false);
const dragSelectStartPos = ref<Vector2d>();
const dragSelectEndPos = ref<Vector2d>();
const dragSelectPreviewKeys = ref<DiagramElementUniqueKey[]>([]);
const SELECTION_BOX_INNER_COLOR = tinycolor(SELECTION_COLOR)
  .setAlpha(0.4)
  .toRgbString();

function beginDragSelect() {
  if (!containerPointerPos.value) return;
  dragSelectPreviewKeys.value = [];
  dragSelectActive.value = true;
  // this triggers after the user breaks the dragging threshold, so we don't start at current position, but where they clicked
  dragSelectStartPos.value = convertContainerCoordsToGridCoords(
    containerPointerPos.value,
  );
  dragSelectEndPos.value = undefined;
}

function onDragSelectMove() {
  dragSelectEndPos.value = gridPointerPos.value;
}

function endDragSelect(doSelection = true) {
  dragSelectActive.value = false;

  const selectedInBoxKeys: DiagramElementUniqueKey[] = [];
  _.each(viewsStore.groups, (nodeRect, nodeKey) => {
    const rect = { ...nodeRect };
    rect.x -= rect.width / 2;
    const inSelectionBox = checkRectanglesOverlap(
      pointsToRect(dragSelectStartPos.value!, dragSelectEndPos.value!),
      rect,
    );
    if (inSelectionBox)
      selectedInBoxKeys.push(DiagramGroupData.generateUniqueKey(nodeKey));
  });
  _.each(viewsStore.components, (rect, nodeKey) => {
    const inSelectionBox = checkRectanglesOverlap(
      pointsToRect(dragSelectStartPos.value!, dragSelectEndPos.value!),
      rect,
    );
    if (inSelectionBox)
      selectedInBoxKeys.push(DiagramNodeData.generateUniqueKey(nodeKey));
  });
  _.each(viewsStore.viewNodes, (node, nodeKey) => {
    const inSelectionBox = checkRectanglesOverlap(
      pointsToRect(dragSelectStartPos.value!, dragSelectEndPos.value!),
      node.def,
    );
    if (inSelectionBox)
      selectedInBoxKeys.push(DiagramViewData.generateUniqueKey(nodeKey));
  });
  // if holding shift key, we'll add/toggle the existing selection with what's in the box
  // NOTE - weird edge cases around what if you let go of shift after beginning the drag which we are ignoring
  if (lastMouseDownEvent.value?.shiftKey) {
    dragSelectPreviewKeys.value = _.xorBy(
      currentSelectionKeys.value,
      selectedInBoxKeys,
    );
  } else {
    dragSelectPreviewKeys.value = selectedInBoxKeys;
  }

  // if option key was held to force drag select, we ignore the element clicked and any parents
  if (lastMouseDownEvent.value?.altKey) {
    if (
      lastMouseDownElement.value &&
      "componentId" in lastMouseDownElement.value.def
    ) {
      const ignoreKeys = [
        lastMouseDownElementKey.value,
        ..._.map(
          lastMouseDownElement.value?.def.ancestorIds,
          getDiagramElementKeyForComponentId,
        ),
      ];

      dragSelectPreviewKeys.value = _.reject(
        dragSelectPreviewKeys.value,
        (key) => ignoreKeys.includes(key),
      );
    }
  }

  if (doSelection) setSelectionByKey(dragSelectPreviewKeys.value);
}

/*
 * MOVING DIAGRAM ELEMENTS (nodes/groups/annotations/etc) ///////////////////////////////////////
 */
const dragElementsActive = ref(false);
const currentSelectionMovableElements = computed(() => {
  // filter selection for nodes and groups
  const elements = _.filter(
    currentSelectionElements.value,
    (el) => el && "componentType" in el.def,
  ) as unknown as (DiagramNodeData | DiagramGroupData | DiagramViewData)[];

  // cannot move elements that are actually gone already
  return elements.filter((e) => {
    if ("changeStatus" in e.def && e.def.changeStatus === "deleted")
      return false;
    if ("fromBaseChangeSet" in e.def && e.def.fromBaseChangeSet) return false;
    return true;
  });
});

const findChildrenByBoundingBox = (
  el: DiagramNodeData | DiagramGroupData,
  allowDeletedChildrenToBeFilteredOut: boolean,
): (DiagramNodeData | DiagramGroupData | DiagramViewData)[] => {
  const cRect = el.def.isGroup
    ? viewsStore.groups[el.def.id]
    : viewsStore.components[el.def.id];
  if (!cRect) return [];

  const rect = { ...cRect };
  rect.x -= rect.width / 2;

  const nodes: (DiagramGroupData | DiagramNodeData | DiagramViewData)[] = [];
  const process = ([id, elRect]: [ComponentId, IRect]) => {
    // i do not fit inside myself
    if (el.def.id === id) return;
    const _r = { ...elRect };
    _r.x -= _r.width / 2;
    if (rectContainsAnother(rect, _r)) {
      const component = componentsStore.allComponentsById[id];
      if (component) {
        if (allowDeletedChildrenToBeFilteredOut) {
          if (
            "changeStatus" in component.def &&
            component.def.changeStatus === "deleted"
          )
            return;
          if (
            "fromBaseChangeSet" in component.def &&
            component.def.fromBaseChangeSet
          )
            return;
        }
        nodes.push(component);
      }
    }
  };

  Object.entries(viewsStore.groups).forEach(process);
  Object.entries(viewsStore.components).forEach(process);
  Object.values(viewsStore.viewNodes).forEach((viewNode) => {
    const _r = {
      x: viewNode.def.x,
      y: viewNode.def.y,
      width: viewNode.def.width,
      height: viewNode.def.height,
    };
    _r.x -= _r.width / 2;
    _r.y -= _r.height / 2;
    if (rectContainsAnother(rect, _r)) {
      nodes.push(viewNode);
    }
  });
  return nodes;
};

const draggedElementsPositionsPreDrag = ref<
  Record<DiagramElementUniqueKey, Vector2d | undefined>
>({});
const edgeScrolledDuringDrag = ref<Vector2d>({ x: 0, y: 0 });

const draggedChildren = ref<
  (DiagramNodeData | DiagramGroupData | DiagramViewData)[]
>([]);
function beginDragElements() {
  if (!lastMouseDownElement.value) return;
  dragElementsActive.value = true;

  edgeScrolledDuringDrag.value = { x: 0, y: 0 };

  const children: Set<DiagramNodeData | DiagramGroupData | DiagramViewData> =
    new Set();
  currentSelectionMovableElements.value.forEach((el) => {
    if (el.def.componentType !== ComponentType.View) {
      const childs = findChildrenByBoundingBox(
        el as DiagramNodeData | DiagramGroupData,
        true,
      );
      childs.forEach((c) => children.add(c));
    }
  });
  draggedChildren.value = [...children];

  // starting position of all children and dragging elements
  draggedElementsPositionsPreDrag.value = currentSelectionMovableElements.value
    .concat(draggedChildren.value)
    .reduce((obj, el) => {
      const geo = viewsStore.geoFrom(el);

      if (geo) obj[el.uniqueKey] = { ...geo };
      return obj;
    }, {} as Record<DiagramElementUniqueKey, Vector2d>);
}

function onDragElementsMove() {
  if (!containerPointerPos.value) return;
  if (!lastMouseDownContainerPointerPos.value) return;

  // this is the max delta from the original mouse down point
  const delta: Vector2d = {
    x: Math.round(
      (containerPointerPos.value.x -
        lastMouseDownContainerPointerPos.value.x +
        edgeScrolledDuringDrag.value.x) /
        zoomLevel.value,
    ),
    y: Math.round(
      (containerPointerPos.value.y -
        lastMouseDownContainerPointerPos.value.y +
        edgeScrolledDuringDrag.value.y) /
        zoomLevel.value,
    ),
  };

  // if shift key is down, we only move on one axis (whichever delta is largest)
  if (shiftKeyIsDown.value) {
    const absDelta = { x: Math.abs(delta.x), y: Math.abs(delta.y) };
    if (absDelta.x > absDelta.y) delta.y = 0;
    else delta.x = 0;
  }

  const parentOrCandidate = allElementsByKey.value[
    cursorWithinGroupKey.value || ""
  ] as DiagramGroupData;

  const adjust: Vector2d = { x: 0, y: 0 };
  if (parentOrCandidate) {
    currentSelectionMovableElements.value.forEach((el) => {
      if (!draggedElementsPositionsPreDrag.value?.[el.uniqueKey]) return;
      const newPosition = vectorAdd(
        draggedElementsPositionsPreDrag.value[el.uniqueKey]!,
        delta,
      );

      // if we are going to move the element within a new parent we may need to adjust
      // the position to stay inside of it
      const parentRect = viewsStore.groups[parentOrCandidate.def.id];
      const elRect = viewsStore.geoFrom(el);
      if (!parentRect || !elRect) return;
      const movedElRect = {
        x: newPosition.x - elRect.width / 2,
        y: newPosition.y,
        width: elRect.width,
        height: elRect.height,
      };

      const parentRectWithBuffer = {
        x: parentRect.x + GROUP_INTERNAL_PADDING - parentRect.width / 2,
        y: parentRect.y + GROUP_INTERNAL_PADDING,
        width: parentRect.width - GROUP_INTERNAL_PADDING * 2,
        height:
          parentRect.height -
          GROUP_INTERNAL_PADDING -
          GROUP_BOTTOM_INTERNAL_PADDING,
      };

      if (!rectContainsAnother(parentRectWithBuffer, movedElRect)) {
        const _adjust = getAdjustmentRectToContainAnother(
          parentRectWithBuffer,
          movedElRect,
        );
        adjust.x = _adjust.x * -1;
        adjust.y = _adjust.y * -1;
      }
    });
  }

  const result = vectorAdd(delta, adjust);
  // when we update the stores, we don't want to use the max delta
  // i just need the difference between the last and this, to move the elements
  const deltaFromLast = vectorSubtract(result, prevDragTotal.value);
  // but keep the total vector for the next iteration through the loop!
  prevDragTotal.value = { ...result };

  const selectionIds = currentSelectionMovableElements.value.map(
    (s) => s.def.id,
  );
  const _components: (DiagramGroupData | DiagramNodeData)[] = [];
  const _views: DiagramViewData[] = [];
  [
    ...currentSelectionMovableElements.value.concat(
      draggedChildren.value.filter((c) => !selectionIds.includes(c.def.id)),
    ),
  ].forEach((c) => {
    if (c.def.componentType === ComponentType.View)
      _views.push(c as DiagramViewData);
    else _components.push(c as DiagramGroupData | DiagramNodeData);
  });
  if (_components.length > 0)
    viewsStore.MOVE_COMPONENTS(_components, deltaFromLast, {
      broadcastToClients: true,
    });
  if (_views.length > 0)
    viewsStore.MOVE_VIEWS(_views, deltaFromLast, {
      broadcastToClients: true,
    });

  checkDiagramEdgeForScroll();
}

function endDragElements() {
  dragElementsActive.value = false;
  prevDragTotal.value = { x: 0, y: 0 };
  const selectionIds = currentSelectionMovableElements.value.map(
    (s) => s.def.id,
  );
  const movedComponents = [
    ...currentSelectionMovableElements.value.concat(
      draggedChildren.value.filter((c) => !selectionIds.includes(c.def.id)),
    ),
  ];
  const _components: (DiagramGroupData | DiagramNodeData)[] = [];
  const _views: DiagramViewData[] = [];
  movedComponents.forEach((c) => {
    if (c.def.componentType === ComponentType.View)
      _views.push(c as DiagramViewData);
    else _components.push(c as DiagramGroupData | DiagramNodeData);
  });

  const detach = !cursorWithinGroupKey.value;
  let newParent: DiagramGroupData | undefined;
  if (cursorWithinGroupKey.value) {
    newParent =
      componentsStore.groupsById[cursorWithinGroupKey.value.substring(2)];
  }

  const nonChildElements = currentSelectionMovableElements.value.filter(
    (component) => {
      if (draggedChildren.value.length === 0) return true;

      const idx = draggedChildren.value.findIndex(
        (child) => child.def.id === component.def.id,
      );
      return idx === -1;
    },
  );

  // note that this is a set which does `===` equality, which for objec
  const setParents: Record<ComponentId, DiagramNodeData | DiagramGroupData> =
    {};
  nonChildElements.forEach((component) => {
    if (!("parentId" in component.def)) return;
    // if their current parent is NOT in this view, do not re-parent!!!
    if (
      component.def.parentId &&
      !Object.keys(viewsStore.groups).includes(component.def.parentId)
    )
      return;

    // views dont have parents
    if (component instanceof DiagramViewData) return;
    // no parent, no call needed
    if (!component.def.parentId && detach) return;
    // same parent, no call needed
    if (component.def.parentId === newParent?.def.id) return;

    if (component.def.parentId && detach)
      setParents[component.def.id] = component;

    if (!component.def.parentId && newParent?.def.id)
      setParents[component.def.id] = component;

    if (component.def.parentId !== newParent?.def.id)
      setParents[component.def.id] = component;
  });

  if (Object.keys(setParents).length > 0) {
    viewsStore.SET_PARENT(Object.keys(setParents), newParent?.def.id ?? null);
  }

  // do i need to resize the new parent to fit the children?
  const parentSize = structuredClone(
    toRaw(viewsStore.groups[newParent?.def.id || ""]),
  );
  if (parentSize && newParent && Object.values(setParents).length > 0) {
    parentSize.x -= parentSize.width / 2;
    const newSize: Partial<Bounds> = {};
    Object.values(setParents).forEach((el) => {
      const isGroup = el.def.componentType !== ComponentType.Component;
      const geo = structuredClone(
        toRaw(
          isGroup
            ? viewsStore.groups[el.def.id]
            : viewsStore.components[el.def.id],
        ),
      );
      if (!geo) return;

      geo.x -= geo.width / 2;

      if (!newSize.left || geo.x < newSize.left) newSize.left = geo.x;
      if (!newSize.top || geo.y < newSize.top)
        newSize.top = geo.y - (isGroup ? NODE_HEADER_HEIGHT * 2 : 0);
      const right = geo.x + geo.width;
      const bottom = geo.y + geo.height;
      if (!newSize.right || right > newSize.right) newSize.right = right;
      if (!newSize.bottom || bottom > newSize.bottom) newSize.bottom = bottom;
    });
    const bounds = toRequiredBounds(newSize); // removes the Partial
    const newRect: IRect = {
      x: bounds.left,
      y: bounds.top,
      width: bounds.right - bounds.left,
      height: bounds.bottom - bounds.top,
    };

    // does the parent needs to be larger?
    if (!rectContainsAnother(parentSize, newRect)) {
      const DEFAULT_GUTTER_SIZE = 10; // leaving room for sockets

      newRect.width += DEFAULT_GUTTER_SIZE * 6;
      newRect.height += HEADER_SIZE;
      newRect.height += DEFAULT_GUTTER_SIZE * 4;

      // we need just a bit more padding space between the parent to fix resizability
      newRect.height += 30;

      // don't make the parent smaller
      if (
        newRect.width * newRect.height >
        parentSize.width * parentSize.height
      ) {
        newRect.x += newRect.width / 2;
        viewsStore.RESIZE_COMPONENT(newParent, newRect, {
          writeToChangeSet: true,
          broadcastToClients: true,
        });
      } else {
        try {
          throw new Error(
            `We prevented making a parent smaller ${JSON.stringify(
              parentSize,
            )} vs ${JSON.stringify(newRect)}`,
          );
        } catch (e) {
          reportError(e);
        }
      }
    }
  }

  if (_components.length > 0)
    viewsStore.MOVE_COMPONENTS(
      _components,
      { x: 0, y: 0 },
      { writeToChangeSet: true },
    );
  if (_views.length > 0)
    viewsStore.MOVE_VIEWS(_views, { x: 0, y: 0 }, { writeToChangeSet: true });
  draggedChildren.value = [];
}

let dragToEdgeScrollInterval: ReturnType<typeof setInterval> | undefined;

function checkDiagramEdgeForScroll() {
  // check if dragging to the edge of the screen, which will trigger scrolling
  if (!containerPointerPos.value) return;
  const pointerX = containerPointerPos.value.x;
  const pointerY = containerPointerPos.value.y;
  if (
    pointerX <= DRAG_EDGE_TRIGGER_SCROLL_WIDTH ||
    pointerY <= DRAG_EDGE_TRIGGER_SCROLL_WIDTH ||
    pointerX >= containerWidth.value - DRAG_EDGE_TRIGGER_SCROLL_WIDTH ||
    pointerY >= containerHeight.value - DRAG_EDGE_TRIGGER_SCROLL_WIDTH
  ) {
    if (!dragToEdgeScrollInterval) {
      dragToEdgeScrollInterval = setInterval(triggerDragToEdgeScrolling, 50);
    }
  } else {
    clearInterval(dragToEdgeScrollInterval!);
    dragToEdgeScrollInterval = undefined;
  }
}

onBeforeUnmount(() => {
  if (dragToEdgeScrollInterval) clearInterval(dragToEdgeScrollInterval);
});

function triggerDragToEdgeScrolling() {
  if (!containerPointerPos.value) return;
  const pointerX = containerPointerPos.value.x;
  const pointerY = containerPointerPos.value.y;

  // determine which direction(s) we need to scroll
  // scroll intensity ramps up as pointer gets closer to edge, but has a maximum
  let deltaX = 0;
  let deltaY = 0;
  if (pointerX <= DRAG_EDGE_TRIGGER_SCROLL_WIDTH) {
    deltaX = Math.min(
      -DRAG_EDGE_TRIGGER_SCROLL_WIDTH,
      pointerX - DRAG_EDGE_TRIGGER_SCROLL_WIDTH,
    );
  } else if (
    pointerX >=
    containerWidth.value - DRAG_EDGE_TRIGGER_SCROLL_WIDTH
  ) {
    deltaX = Math.max(
      DRAG_EDGE_TRIGGER_SCROLL_WIDTH,
      pointerX - (containerWidth.value - DRAG_EDGE_TRIGGER_SCROLL_WIDTH),
    );
  }

  if (pointerY <= DRAG_EDGE_TRIGGER_SCROLL_WIDTH) {
    deltaY = Math.min(
      -DRAG_EDGE_TRIGGER_SCROLL_WIDTH,
      pointerY - DRAG_EDGE_TRIGGER_SCROLL_WIDTH,
    );
  } else if (
    pointerY >=
    containerHeight.value - DRAG_EDGE_TRIGGER_SCROLL_WIDTH
  ) {
    deltaY = Math.max(
      DRAG_EDGE_TRIGGER_SCROLL_WIDTH,
      pointerY - (containerHeight.value - DRAG_EDGE_TRIGGER_SCROLL_WIDTH),
    );
  }

  // track total amount scrolled because we need to offset from original drag click location
  edgeScrolledDuringDrag.value.x += deltaX;
  edgeScrolledDuringDrag.value.y += deltaY;

  // adjust amount to scroll by zoom before we apply it
  if (deltaX !== 0) deltaX /= zoomLevel.value;
  if (deltaY !== 0) deltaY /= zoomLevel.value;

  gridOrigin.value = {
    x: gridOrigin.value.x + deltaX,
    y: gridOrigin.value.y + deltaY,
  };

  // call mouse move handler which actually moves the elements
  onDragElementsMove();
}

function alignSelection(direction: Direction) {
  if (!currentSelectionMovableElements.value.length) return;
  let alignedX: number | undefined;
  let alignedY: number | undefined;
  const positions = _.map(currentSelectionMovableElements.value, (el) =>
    viewsStore.geoFrom(el),
  ).filter(nonNullable);
  const xPositions = _.map(positions, (p) => p.x);
  const yPositions = _.map(positions, (p) => p.y);
  if (direction === "up") alignedY = _.min(yPositions);
  else if (direction === "down") alignedY = _.max(yPositions);
  else if (direction === "left") alignedX = _.min(xPositions);
  else if (direction === "right") alignedX = _.max(xPositions);

  const _components: (DiagramGroupData | DiagramNodeData)[] = [];
  const _views: DiagramViewData[] = [];
  currentSelectionMovableElements.value.forEach((c) => {
    if (c.def.componentType === ComponentType.View)
      _views.push(c as DiagramViewData);
    else _components.push(c as DiagramGroupData | DiagramNodeData);
  });
  if (_components.length)
    viewsStore.MOVE_COMPONENTS(
      _components,
      { x: alignedX ?? 0, y: alignedY ?? 0 },
      { writeToChangeSet: true },
    );
  if (_views.length)
    viewsStore.MOVE_VIEWS(
      _views,
      { x: alignedX ?? 0, y: alignedY ?? 0 },
      { writeToChangeSet: true },
    );
}

type VoidFn = () => void;
let debouncedNudgeFn: _.DebouncedFunc<VoidFn> | null;
let debouncedNudgeFnViews: _.DebouncedFunc<VoidFn> | null;
function nudgeSelection(direction: Direction, largeNudge: boolean) {
  if (!currentSelectionMovableElements.value.length) return;
  const nudgeSize = largeNudge ? 10 : 1;
  const nudgeVector: Vector2d = {
    left: { x: -1 * nudgeSize, y: 0 },
    right: { x: 1 * nudgeSize, y: 0 },
    up: { x: 0, y: -1 * nudgeSize },
    down: { x: 0, y: 1 * nudgeSize },
  }[direction];

  const _components: (DiagramGroupData | DiagramNodeData)[] = [];
  const _views: DiagramViewData[] = [];
  currentSelectionMovableElements.value.forEach((c) => {
    if (c.def.componentType === ComponentType.View)
      _views.push(c as DiagramViewData);
    else _components.push(c as DiagramGroupData | DiagramNodeData);
  });

  if (_components.length > 0)
    viewsStore.MOVE_COMPONENTS(_components, nudgeVector, {
      broadcastToClients: true,
    });
  if (!debouncedNudgeFn && _components.length > 0) {
    debouncedNudgeFn = _.debounce(() => {
      viewsStore.MOVE_COMPONENTS(
        _components,
        { x: 0, y: 0 },
        { writeToChangeSet: true },
      );
      debouncedNudgeFn = null;
    }, 300);
    debouncedNudgeFn();
  }

  if (_views.length > 0)
    viewsStore.MOVE_VIEWS(_views, nudgeVector, {
      broadcastToClients: true,
    });
  if (!debouncedNudgeFnViews && _views.length > 0) {
    debouncedNudgeFnViews = _.debounce(() => {
      viewsStore.MOVE_VIEWS(_views, { x: 0, y: 0 }, { writeToChangeSet: true });
      debouncedNudgeFn = null;
    }, 300);
    debouncedNudgeFnViews();
  }
}

// we calculate which group (if any) the cursor is within without using hover events
// which is useful when dragging elements in/out of groups
const cursorWithinGroupKey = computed(() => {
  // groups are sorted by depth so they render in the right order
  // so we search from the opposite direction to find deepest child first
  if (!gridPointerPos.value) return undefined;

  const withinGroup = _.findLast(groups.value, (group) => {
    // skip groups that are selected
    if (currentSelectionKeys.value.includes(group.uniqueKey)) return false;

    const frameRect = viewsStore.groups[group.def.id];
    if (!frameRect) {
      return false;
    }
    const geo = { ...frameRect };
    geo.x -= geo.width / 2;
    return rectContainsPoint(geo, gridPointerPos.value!);
  });
  return withinGroup?.uniqueKey;
});

const moveElementsState = computed(() => {
  return {
    active: dragElementsActive.value,
    intoNewParentKey: cursorWithinGroupKey.value,
  } as MoveElementsState;
});

// RESIZING DIAGRAM ELEMENTS (groups) ///////////////////////////////////////
const resizeElement = ref<DiagramGroupData>();
const resizeElementActive = computed(() => !!resizeElement.value);
const resizeElementDirection = ref<SideAndCornerIdentifiers>();
const resizedElementGeometryPreResize = ref<IRect>();

function beginResizeElement() {
  if (!lastMouseDownElement.value) return;
  if (lastMouseDownHoverMeta.value?.type !== "resize") return;

  resizeElementDirection.value = lastMouseDownHoverMeta.value.direction;

  const irect = viewsStore.groups[lastMouseDownElement.value.def.id];
  if (!irect) return; // not a group

  resizeElement.value = lastMouseDownElement.value as DiagramGroupData;
  resizedElementGeometryPreResize.value = { ...irect };
}

function endResizeElement() {
  resizedElementGeometryPreResize.value = undefined;
  const el = resizeElement.value;
  if (!el) return;

  const geometry = viewsStore.groups[el.def.id];
  if (!geometry) {
    return;
  }
  viewsStore.RESIZE_COMPONENT(el, geometry, {
    writeToChangeSet: true,
  });

  resizeElement.value = undefined;
}

function onResizeMove() {
  if (!resizeElement.value || !resizeElementDirection.value) return;

  if (!containerPointerPos.value) return;
  if (!lastMouseDownContainerPointerPos.value) return;

  const sizeDelta: Vector2d = {
    x: Math.round(
      (containerPointerPos.value.x -
        lastMouseDownContainerPointerPos.value.x +
        edgeScrolledDuringDrag.value.x) /
        zoomLevel.value,
    ),
    y: Math.round(
      (containerPointerPos.value.y -
        lastMouseDownContainerPointerPos.value.y +
        edgeScrolledDuringDrag.value.y) /
        zoomLevel.value,
    ),
  };

  const positionDelta: Vector2d = {
    x: 0,
    y: 0,
  };

  if (!resizedElementGeometryPreResize.value) {
    return;
  }

  const rightBound =
    resizedElementGeometryPreResize.value.x +
    resizedElementGeometryPreResize.value.width / 2;

  // Ensure the component never gets smaller than its minimum dimensions
  switch (resizeElementDirection.value) {
    case "bottom":
      {
        sizeDelta.x = 0;
        const minDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.height;
        if (sizeDelta.y < minDelta) {
          sizeDelta.y = minDelta;
        }
        if (
          resizedElementGeometryPreResize.value.height + sizeDelta.y <
          resizeElement.value.socketEndingY
        ) {
          sizeDelta.y =
            resizeElement.value.socketEndingY -
            resizedElementGeometryPreResize.value.height;
        }
      }
      break;
    case "top":
      {
        sizeDelta.x = 0;
        sizeDelta.y = -sizeDelta.y;
        const minDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.height;
        if (sizeDelta.y < minDelta) {
          sizeDelta.y = minDelta;
        }
        positionDelta.y = -sizeDelta.y;
      }
      break;
    case "left":
      {
        sizeDelta.y = 0;
        sizeDelta.x = -sizeDelta.x;
        const minDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.width;
        if (sizeDelta.x < minDelta) {
          sizeDelta.x = minDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "right":
      {
        sizeDelta.y = 0;
        const minDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.width;
        if (sizeDelta.x < minDelta) {
          sizeDelta.x = minDelta;
        }
        positionDelta.x = sizeDelta.x;
      }
      break;
    case "bottom-left":
      {
        const minYDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }

        sizeDelta.x = -sizeDelta.x;
        const minXDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "bottom-right":
      {
        const minYDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }
        const minXDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = sizeDelta.x;
      }
      break;
    case "top-left":
      {
        sizeDelta.y = -sizeDelta.y;
        const minYDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }
        positionDelta.y = -sizeDelta.y;

        sizeDelta.x = -sizeDelta.x;
        const minXDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "top-right":
      {
        sizeDelta.y = -sizeDelta.y;
        const minYDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }
        positionDelta.y = -sizeDelta.y;

        const minXDelta =
          MIN_NODE_DIMENSION - resizedElementGeometryPreResize.value.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = sizeDelta.x;
      }
      break;
    default:
      break;
  }

  const newNodeSize = {
    width: resizedElementGeometryPreResize.value.width + sizeDelta.x,
    height: resizedElementGeometryPreResize.value.height + sizeDelta.y,
  };

  // Get the correctly cached position for the element being resized
  const newNodePosition = {
    x: resizedElementGeometryPreResize.value.x + positionDelta.x / 2,
    y: resizedElementGeometryPreResize.value.y + positionDelta.y,
  };

  // Make sure the frame doesn't shrink to be smaller than it's children
  const contentsBox =
    viewsStore.contentBoundingBoxesByGroupId[resizeElement.value.def.id];

  if (contentsBox) {
    // Resized element with top-left corner xy coordinates instead of top-center
    const newNodeRect = {
      ...newNodePosition,
      ...newNodeSize,
      x: newNodePosition.x - newNodeSize.width / 2,
    };

    // if resized was going to get smaller than children bounding box, set it to minimum necessary dimensions
    {
      const contentBottomY = contentsBox.y + contentsBox.height;
      const minimumAcceptedHeight = contentBottomY - newNodeRect.y;
      newNodeRect.height = Math.round(
        Math.max(newNodeSize.height, minimumAcceptedHeight),
      );

      // handle resizing from the top
      if (newNodeRect.y > contentsBox.y) {
        newNodeRect.y = contentsBox.y;
        newNodeRect.height =
          resizedElementGeometryPreResize.value.y +
          resizedElementGeometryPreResize.value.height -
          contentsBox.y;
      }
    }

    // Check right collision
    {
      const internalX = contentsBox.x + contentsBox.width;
      const externalX = newNodeRect.x + newNodeRect.width;
      if (internalX > externalX) {
        const minimumWidth = internalX - newNodeRect.x;
        newNodeRect.width = minimumWidth;
      }
    }

    // Check left collision
    {
      const internalX = contentsBox.x;
      const externalX = newNodeRect.x;

      if (internalX < externalX) {
        newNodeRect.x = internalX;
        newNodeRect.width = rightBound - newNodeRect.x;
      }
    }

    newNodePosition.x = newNodeRect.x + newNodeRect.width / 2;
    newNodePosition.y = newNodeRect.y;
    newNodeSize.width = newNodeRect.width;
    newNodeSize.height = newNodeRect.height;
  }

  // Make sure the frame doesn't get larger than parent
  const parentGeometry =
    viewsStore.groups[resizeElement.value.def.parentId || ""];

  if (parentGeometry) {
    // Resized element with top-left corner xy coordinates instead of top-center
    const newNodeRect = {
      ...newNodePosition,
      ...newNodeSize,
      x: newNodePosition.x - newNodeSize.width / 2,
    };

    // Unsure I need these constants
    const parentContentRect = {
      x: parentGeometry.x - parentGeometry.width / 2 + GROUP_INTERNAL_PADDING,
      y: parentGeometry.y + GROUP_INTERNAL_PADDING,
      width: parentGeometry.width - GROUP_INTERNAL_PADDING * 2,
      height:
        parentGeometry.height -
        GROUP_INTERNAL_PADDING -
        GROUP_BOTTOM_INTERNAL_PADDING,
    };

    // Top Collision
    if (parentContentRect.y > newNodeRect.y - GROUP_INNER_Y_BOUNDARY_OFFSET) {
      newNodeRect.y = parentContentRect.y + GROUP_INNER_Y_BOUNDARY_OFFSET;
      newNodeRect.height =
        resizedElementGeometryPreResize.value.y +
        resizedElementGeometryPreResize.value.height -
        parentContentRect.y -
        GROUP_INNER_Y_BOUNDARY_OFFSET;
    }

    // Bottom collision
    const bottom = parentContentRect.y + parentContentRect.height;
    if (bottom < newNodeRect.y + newNodeRect.height) {
      newNodeRect.height = bottom - newNodeRect.y;
    }

    // Right collision
    const parentRight = parentContentRect.x + parentContentRect.width;
    const childRight = newNodeRect.x + newNodeRect.width;
    if (childRight > parentRight) {
      newNodeRect.width = parentRight - newNodeRect.x;
    }

    // Left collision
    const parentLeft = parentContentRect.x;
    const childLeft = newNodeRect.x;
    if (childLeft < parentLeft) {
      newNodeRect.x = parentLeft;
      newNodeRect.width = rightBound - parentLeft;
    }

    newNodePosition.x = newNodeRect.x + newNodeRect.width / 2;
    newNodePosition.y = newNodeRect.y;
    newNodeSize.width = newNodeRect.width;
    newNodeSize.height = newNodeRect.height;
  }

  viewsStore.RESIZE_COMPONENT(
    resizeElement.value,
    { ...newNodePosition, ...newNodeSize },
    { broadcastToClients: true },
  );
}

// DRAWING EDGES ///////////////////////////////////////////////////////////////////////
const drawEdgeActive = ref(false);
const drawEdgeFromSocketKey = ref<DiagramElementUniqueKey>();
const drawEdgeFromSocket = computed(
  () => getElementByKey(drawEdgeFromSocketKey.value) as DiagramSocketData,
);
const drawEdgeToSocketKey = ref<DiagramElementUniqueKey>();
const drawEdgeToSocket = computed(
  () => getElementByKey(drawEdgeToSocketKey.value) as DiagramSocketData,
);
const drawEdgePossibleTargetSocketKeys = computed(() => {
  if (!drawEdgeActive.value) return [];

  const fromSocket = drawEdgeFromSocket.value;
  if (!fromSocket) return [];

  const allExistingEdges =
    connectedEdgesByElementKey.value[fromSocket.uniqueKey];
  const actualExistingEdges = _.reject(
    allExistingEdges,
    (e) => e.def.changeStatus === "deleted",
  );
  const existingConnectedSocketKeys = _.map(actualExistingEdges, (edge) =>
    edge.fromSocketKey === fromSocket.uniqueKey
      ? edge.toSocketKey
      : edge.fromSocketKey,
  );
  const possibleSockets = _.filter(sockets.value, (possibleToSocket) => {
    // cannot connect sockets to other sockets on same node (at least not currently)
    if (possibleToSocket.parent === fromSocket.parent) return false;
    // cannot connect to a socket that is already connected
    if (existingConnectedSocketKeys.includes(possibleToSocket.uniqueKey))
      return false;
    // inputs must be connected to outputs (or bidirectional sockets)
    if (fromSocket.def.direction === possibleToSocket.def.direction)
      return false;

    const isManagedSchema =
      possibleToSocket.def.schemaId &&
      fromSocket.def.managedSchemas &&
      fromSocket.def.managedSchemas.includes(possibleToSocket.def.schemaId);
    const isSameSchema =
      possibleToSocket.def.schemaId === fromSocket.def.schemaId;

    if (fromSocket.def.isManagement && possibleToSocket.def.isManagement) {
      return !!(isSameSchema || isManagedSchema);
    }

    const [outputCAs, inputCAs] =
      fromSocket.def.direction === "output"
        ? [
            fromSocket.def.connectionAnnotations,
            possibleToSocket.def.connectionAnnotations,
          ]
        : [
            possibleToSocket.def.connectionAnnotations,
            fromSocket.def.connectionAnnotations,
          ];

    // check socket connection annotations compatibility
    for (const outputCA of outputCAs) {
      for (const inputCA of inputCAs) {
        if (connectionAnnotationFitsReference(outputCA, inputCA)) {
          return true;
        }
      }
    }

    return false;
  });
  return _.map(possibleSockets, (s) => s.uniqueKey);
});

const drawEdgeWillDeleteEdges = computed(() => {
  if (!drawEdgeActive.value) return [];
  const fromSocket = drawEdgeFromSocket.value;
  const toSocket = drawEdgeToSocket.value;
  if (!fromSocket) return [];

  // there will/should always be a fromSocket if draw edge is active

  const edgesToDelete = [] as DiagramEdgeData[];
  // currently we only care about arity of 1 or N - but this logic would need to change to support arity of a specific number
  if (fromSocket.def.maxConnections !== null) {
    edgesToDelete.push(
      ...(connectedEdgesByElementKey.value[fromSocket.uniqueKey] || []),
    );
  }

  if (toSocket && toSocket.def.maxConnections !== null) {
    edgesToDelete.push(
      ...(connectedEdgesByElementKey.value[toSocket.uniqueKey] || []),
    );
  }
  return _.reject(edgesToDelete, (e) => e.def.changeStatus === "deleted");
});

const drawEdgeState = computed(() => {
  return {
    active: drawEdgeActive.value,
    fromSocketKey: drawEdgeFromSocketKey.value,
    toSocketKey: drawEdgeToSocketKey.value,
    possibleTargetSocketKeys: drawEdgePossibleTargetSocketKeys.value,
    edgeKeysToDelete: _.map(drawEdgeWillDeleteEdges.value, (e) => e.uniqueKey),
  } as DiagramDrawEdgeState;
});

function beginDrawEdge(fromSocket: DiagramSocketData) {
  drawEdgeActive.value = true;
  drawEdgeFromSocketKey.value = fromSocket.uniqueKey;
  drawEdgeToSocketKey.value = undefined;
}

function onDrawEdgeMove() {
  if (!gridPointerPos.value) return;
  // look through the possible target sockets, and find distances to the pointer
  const socketPointerDistances = _.map(
    drawEdgePossibleTargetSocketKeys.value,
    (socketKey) => {
      const socketLocation = viewsStore.sockets[socketKey];
      // Not sure what this should do if we can't find a location
      const center = socketLocation?.center ?? { x: 0, y: 0 };
      return {
        socketKey,
        pointerDistance: vectorDistance(gridPointerPos.value!, center),
      };
    },
  );
  const nearest = _.minBy(socketPointerDistances, (d) => d.pointerDistance);
  // give a little buffer so the pointer will magnet to nearby sockets
  if (nearest && nearest.pointerDistance < SOCKET_SIZE * 2) {
    drawEdgeToSocketKey.value = nearest.socketKey;
  } else {
    drawEdgeToSocketKey.value = undefined;
  }

  checkDiagramEdgeForScroll();
}

async function endDrawEdge() {
  drawEdgeActive.value = false;
  if (!drawEdgeFromSocket.value || !drawEdgeToSocket.value) return;

  // if the user dragged from an input to an output, we'll reverse the direction when we fire off the event
  const swapDirection = drawEdgeFromSocket.value.def.direction === "input";
  const adjustedFrom = swapDirection
    ? drawEdgeToSocket.value
    : drawEdgeFromSocket.value;
  const adjustedTo = swapDirection
    ? drawEdgeFromSocket.value
    : drawEdgeToSocket.value;

  const fromComponentId = adjustedFrom.parent.def.id;
  const fromSocketId = adjustedFrom.def.id;
  const toComponentId = adjustedTo.parent.def.id;
  const toSocketId = adjustedTo.def.id;
  const from = {
    componentId: fromComponentId,
    socketId: fromSocketId,
  };
  const to = {
    componentId: toComponentId,
    socketId: toSocketId,
  };

  if (adjustedFrom.def.isManagement) {
    await componentsStore.MANAGE_COMPONENT(from, to);
  } else {
    await componentsStore.CREATE_COMPONENT_CONNECTION(from, to);
  }
}

const pasteElementsActive = computed(() => {
  return (
    componentsStore.copyingFrom && viewsStore.selectedComponentIds.length > 0
  );
});

// TODO: I dont think we need to compute this
// we can do the work directly in the paste function
const currentSelectionEnclosure: Ref<IRect | undefined> = computed(() => {
  const componentIds = viewsStore.selectedComponentIds;

  if (componentIds.length === 0) return;

  let left;
  let top;
  let right;
  let bottom;
  for (const id of componentIds) {
    const geometry = viewsStore.components[id] || viewsStore.groups[id];
    if (!geometry) continue;

    const thisBoundaries = {
      top: geometry.y,
      bottom: geometry.y + geometry.height,
      left: geometry.x - geometry.width / 2,
      right: geometry.x + geometry.width / 2,
    };

    if (!top || thisBoundaries.top < top) top = thisBoundaries.top;
    if (!bottom || thisBoundaries.bottom > bottom)
      bottom = thisBoundaries.bottom;
    if (!left || thisBoundaries.left < left) left = thisBoundaries.left;
    if (!right || thisBoundaries.right > right) right = thisBoundaries.right;
  }

  if (!left || !top || !right || !bottom) return;

  const width = right - left;
  const height = bottom - top;

  return {
    x: left,
    y: top,
    width,
    height,
  };
});

async function triggerPasteElements() {
  if (!pasteElementsActive.value)
    throw new Error("paste element mode must be active");

  const pasteCenter = gridPointerPos.value;
  if (!pasteCenter) throw new Error("Cursor must be in grid to paste element");

  if (!componentsStore.copyingFrom)
    throw new Error("Copy cursor must be in grid to paste element");

  const selectionEnclosure = currentSelectionEnclosure.value;
  if (!selectionEnclosure) {
    // Fix for BUG-710, reset when pasting across views
    componentsStore.copyingFrom = null;
    viewsStore.selectedComponentIds = [];
    return;
  }

  const selectionCenter = {
    x: selectionEnclosure.x + selectionEnclosure.width / 2,
    y: selectionEnclosure.y + selectionEnclosure.height / 2,
  };

  // Displacement Vector between selection center to paste center.
  // When added to each component, moves it to be centered around paste area
  const selectionOffset = vectorBetween(selectionCenter, pasteCenter);

  // How much to move components in the direction of the paste center
  // from 0 - move to center - to 1 - don't move
  const selectionShrinkCoefficient = {
    x: 1,
    y: 1,
  };

  const newParentId =
    allElementsByKey.value[cursorWithinGroupKey.value ?? "-1"]?.def.id;
  let parentContentArea: IRect | undefined;

  // if we're pasting into a new parent, fit the selection area into it first by shrinking and then translating
  if (newParentId) {
    const parentGeometry = viewsStore.groups[newParentId];
    if (!parentGeometry) throw new Error("Couldn't get parent geometry");

    // the x in component geometry is centered, so we need to translate it to represent the top left
    const parentTopLeft = {
      x: parentGeometry.x - parentGeometry.width / 2,
      y: parentGeometry.y,
    };

    parentContentArea = {
      x: parentTopLeft.x + GROUP_INTERNAL_PADDING,
      y: parentTopLeft.y + GROUP_INTERNAL_PADDING,
      width: parentGeometry.width - GROUP_INTERNAL_PADDING * 2,
      height:
        parentGeometry.height -
        (GROUP_INTERNAL_PADDING + GROUP_BOTTOM_INTERNAL_PADDING),
    };

    // Shrink selection
    if (parentContentArea.width < selectionEnclosure.width) {
      selectionShrinkCoefficient.x =
        parentContentArea.width / selectionEnclosure.width;

      selectionOffset.x = parentContentArea.x - selectionEnclosure.x;
    }

    if (parentContentArea.height < selectionEnclosure.height) {
      selectionShrinkCoefficient.y =
        parentContentArea.height / selectionEnclosure.height;

      selectionOffset.y = parentContentArea.y - selectionEnclosure.y;
    }

    // Move selection to be centered around where it was pasted
    const offsetPasteArea = {
      x: selectionEnclosure.x + selectionOffset.x,
      y: selectionEnclosure.y + selectionOffset.y,
      width: selectionEnclosure.width * selectionShrinkCoefficient.x,
      height: selectionEnclosure.height * selectionShrinkCoefficient.y,
    };

    const fitOffset = getAdjustmentRectToContainAnother(
      parentContentArea,
      offsetPasteArea,
    );

    selectionOffset.x -= fitOffset.x;
    selectionOffset.y -= fitOffset.y;
  }

  const pasteTargets = _.map(viewsStore.selectedComponentIds, (id) => {
    const thisGeometry = viewsStore.components[id] || viewsStore.groups[id];

    if (!thisGeometry) throw new Error("Rendered Component not found");

    // If the selection shrunk, move the children to fit
    let finalPosition = vectorAdd(selectionOffset, thisGeometry);
    if (parentContentArea) {
      const shrinkOffset = getAdjustmentRectToContainAnother(
        parentContentArea,
        {
          x: finalPosition.x - thisGeometry.width / 2,
          y: finalPosition.y,
          width: thisGeometry.width,
          height: thisGeometry.height,
        },
      );

      finalPosition = vectorSubtract(finalPosition, shrinkOffset);

      if (thisGeometry.height >= parentContentArea.height) {
        finalPosition.y = parentContentArea.y;
      }
      if (thisGeometry.width >= parentContentArea.width) {
        finalPosition.x = parentContentArea.x + parentContentArea.width / 2;
      }
    }

    return {
      id,
      componentGeometry: {
        ...finalPosition,
        width: thisGeometry.width,
        height: thisGeometry.height,
      },
    };
  });

  componentsStore.copyingFrom = null;

  await viewsStore.PASTE_COMPONENTS(pasteTargets, newParentId);
}

// ELEMENT ADDITION
const insertElementActive = computed(
  () => !!componentsStore.selectedInsertCategoryVariantId,
);

const outlinerAddActive = computed(() => !!viewsStore.addComponentId);
const viewAddActive = computed(() => !!viewsStore.addViewId);

const HEADER_SIZE = 60; // The height of the component header bar; TODO find a better way to detect this
function fitChildInsideParentFrame(
  position: Vector2d,
  size: Size2D,
): [Vector2d, Size2D] {
  // position the component within its parent cleanly
  // there is headerTextHeight.value, but we don't have it because the component doesn't exist yet
  const DEFAULT_GUTTER_SIZE = 10; // leaving room for sockets
  const createAtPosition = { ...position };
  createAtPosition.y += HEADER_SIZE + DEFAULT_GUTTER_SIZE;
  createAtPosition.x += DEFAULT_GUTTER_SIZE;

  const createAtSize = { ...size };
  // this math isn't working exactly as I would expect, but getting the results I want
  createAtSize.width -= DEFAULT_GUTTER_SIZE * 6;
  createAtSize.height -= HEADER_SIZE;
  createAtSize.height -= DEFAULT_GUTTER_SIZE * 4;

  // we need just a bit more padding space between the parent to fix resizability
  createAtPosition.y += 15;
  createAtSize.height -= 30;

  // enforce minimums
  createAtSize.width = Math.max(createAtSize.width, MIN_NODE_DIMENSION);
  createAtSize.height = Math.max(createAtSize.height, MIN_NODE_DIMENSION);

  return [createAtPosition, createAtSize];
}

async function triggerAddViewToView() {
  if (!viewAddActive.value || !viewsStore.addViewId)
    throw new Error("insert element mode must be active");
  if (!gridPointerPos.value)
    throw new Error("Cursor must be in grid to insert element");

  const addingViewId = viewsStore.addViewId;
  viewsStore.addViewId = null;

  const geo = { ...gridPointerPos.value, radius: 250 };

  viewsStore.ADD_VIEW_TO(viewsStore.selectedViewId!, addingViewId, geo);
}

async function triggerAddToView() {
  if (!outlinerAddActive.value)
    throw new Error("insert element mode must be active");
  if (!gridPointerPos.value)
    throw new Error("Cursor must be in grid to insert element");

  const originView = viewsStore.viewsById[viewsStore.outlinerViewId || ""];
  if (!originView) throw new Error("Origin view does not exist");
  const component =
    componentsStore.allComponentsById[viewsStore.addComponentId || ""];
  viewsStore.addComponentId = null;
  if (!component) throw new Error("Adding component does not exist");
  const createAtSize: IRect = component.def.isGroup
    ? originView.groups[component.def.id]!
    : originView.components[component.def.id]!;
  const createAtPosition = gridPointerPos.value;

  const components: Record<ComponentId, IRect> = {};
  components[component.def.id] = { ...createAtSize, ...createAtPosition };

  viewsStore.ADD_TO(
    viewsStore.outlinerViewId!,
    components,
    viewsStore.selectedViewId!,
    false,
  );
}

async function triggerInsertElement() {
  if (!insertElementActive.value)
    throw new Error("insert element mode must be active");
  if (!gridPointerPos.value)
    throw new Error("Cursor must be in grid to insert element");

  if (!componentsStore.selectedInsertCategoryVariantId)
    throw new Error("missing insert selection metadata");

  const insertVariantId = componentsStore.selectedInsertCategoryVariantId;
  componentsStore.selectedInsertCategoryVariantId = null;

  const categoryVariant = componentsStore.categoryVariantById[insertVariantId];
  if (!categoryVariant) return;

  const isFrame =
    categoryVariant.variant.componentType !== ComponentType.Component;

  const parentGroupId: string | undefined = cursorWithinGroupKey.value?.replace(
    "g-",
    "",
  );

  let parentId;
  let createAtSize: Size2D | undefined;
  let createAtPosition = gridPointerPos.value;

  if (isFrame) createAtSize = { width: 500, height: 500 };

  if (parentGroupId) {
    const parentComponent = componentsStore.groupsById[parentGroupId];
    if (
      parentComponent &&
      (parentComponent.def.componentType !== ComponentType.AggregationFrame ||
        insertVariantId === parentComponent.def.schemaVariantId)
    ) {
      parentId = parentGroupId;
    }

    if (parentComponent) {
      const geometry = viewsStore.groups[parentComponent.def.id];
      if (
        parentComponent?.def.childIds &&
        parentComponent.def.childIds?.length > 0
      ) {
        // when there are already children we can't be as smart
        // leave position as the cursor
        // backend default is 500 x 500, just make it smaller since there are other children
        createAtSize = { width: 250, height: 250 };
      } else if (isFrame && geometry) {
        [createAtPosition, createAtSize] = fitChildInsideParentFrame(
          { x: geometry.x, y: geometry.y },
          { width: geometry.width, height: geometry.height },
        );
      }
    }
  }

  // as this stands, the client will send a width/height for non-frames, that the API endpoint ignores
  // TODO: is there is a good way to determine whether this schemaID is a frame?
  viewsStore.CREATE_COMPONENT(
    insertVariantId,
    createAtPosition,
    parentId,
    createAtSize,
  );
}

// LAYOUT REGISTRY + HELPERS ///////////////////////////////////////////////////////////

type DIRECTION = "to" | "from";
function getSocketLocationInfo(
  direction?: DIRECTION,
  edge?: DiagramEdgeData,
  socketKey?: DiagramElementUniqueKey,
) {
  if (edge) {
    // if from component is collapsed, return the position of its center
    const key = direction === "from" ? edge.fromSocketKey : edge.toSocketKey;
    return viewsStore.sockets[key];
  }

  if (!socketKey) return undefined;
  return viewsStore.sockets[socketKey];
}

// DIAGRAM CONTENTS HELPERS //////////////////////////////////////////////////

const nodes = computed(() => {
  const componentIds = Object.keys(viewsStore.components);
  const components: DiagramNodeData[] = [];
  componentIds.forEach((id) => {
    const c = componentsStore.nodesById[id];
    if (c) components.push(c);
  });
  return components;
});

const groups = computed(() => {
  // order groups biggest at the back, smallest at the front (not according to lineage)
  const componentIds = Object.keys(viewsStore.groups);
  const frames = Object.values(componentsStore.groupsById).filter((g) =>
    componentIds.includes(g.def.id),
  );
  const ancestryByBounds = new DefaultMap<ComponentId, ComponentId[]>(() => []);
  frames.forEach((g) => {
    const childIds = findChildrenByBoundingBox(g, false).map((el) => el.def.id);
    childIds.forEach((child) => {
      const ancestors = ancestryByBounds.get(child);
      ancestors.push(g.def.id);
      ancestryByBounds.set(child, ancestors);
    });
  });
  const orderedGroups = _.orderBy(frames, (g) => {
    const viewGroup = viewsStore.groups[g.def.id]!;
    let zIndex = viewGroup.zIndex;
    // if being dragged (or ancestor being dragged), bump up to front, but maintain order within that frame
    if (
      dragElementsActive.value ||
      viewsStore.selectedComponentIds.length > 0
    ) {
      if (
        _.intersection(
          [g.def.componentId, ...ancestryByBounds.get(g.def.componentId)],
          viewsStore.selectedComponentIds,
        ).length
      ) {
        zIndex += 1000;
      }
    }
    return zIndex;
  });

  return orderedGroups;
});

const sockets = computed(() => {
  const elements = _.concat(nodes.value, groups.value);
  return _.compact(_.flatMap(elements, (i) => i.sockets));
});

// this will re-compute on every drag until all the position data is removed
const allElementsByKey = computed(() =>
  _.keyBy(
    [
      ...nodes.value,
      ...groups.value,
      ...sockets.value,
      ...viewsStore.edges,
      ...Object.values(viewsStore.viewNodes),
    ],
    (e) => e.uniqueKey,
  ),
);

function getElementByKey(key?: DiagramElementUniqueKey) {
  return key ? allElementsByKey.value[key] : undefined;
}

// Selection rects
const selectionRects = computed(() => {
  const rects = [] as (Size2D & Vector2d)[];
  currentSelectionKeys.value.forEach((uniqueKey) => {
    const isView = uniqueKey.startsWith("v-");
    const isGroup = uniqueKey.startsWith("g-");
    const id = uniqueKey.slice(2); // remove the prefix
    if (isView) {
      const rect = viewsStore.viewNodes[id]?.def;
      if (rect) {
        const r = {
          x: rect.x - rect.width / 2,
          y: rect.y - rect.height / 2,
          width: rect.width,
          height: rect.height,
        };
        rects.push(r);
      }
    } else {
      const rect = viewsStore.components[id] || viewsStore.groups[id];
      if (rect) {
        const r = {
          x: rect.x - rect.width / 2,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        };
        if (isGroup) {
          // deal with top bar height outside the component's
          // designated height
          const adjust = 28 + GROUP_HEADER_BOTTOM_MARGIN * 2;
          r.height += adjust;
          r.y -= adjust;
        }
        rects.push(r);
      }
    }
  });
  return rects;
});

function getDiagramElementKeyForComponentId(
  componentId?: ComponentId | null,
): string | undefined {
  if (!componentId) return;
  let component:
    | DiagramNodeData
    | DiagramGroupData
    | DiagramViewData
    | undefined = componentsStore.allComponentsById[componentId];
  if (!component) component = viewsStore.viewNodes[componentId];
  return component?.uniqueKey;
}

function getDiagramElementKeyForEdgeId(edgeId?: EdgeId | null) {
  if (!edgeId) return;
  return DiagramEdgeData.generateUniqueKey(edgeId);
}

const connectedEdgesByElementKey = computed(() => {
  const lookup: Record<DiagramElementUniqueKey, DiagramEdgeData[]> = {};
  Object.values(componentsStore.diagramEdgesById).forEach((edge) => {
    lookup[edge.fromNodeKey] ||= [];
    lookup[edge.fromNodeKey]!.push(edge);
    lookup[edge.toNodeKey] ||= [];
    lookup[edge.toNodeKey]!.push(edge);
    lookup[edge.fromSocketKey] ||= [];
    lookup[edge.fromSocketKey]!.push(edge);
    lookup[edge.toSocketKey] ||= [];
    lookup[edge.toSocketKey]!.push(edge);
  });
  return lookup;
});

// function recenter() {
//   gridOrigin.value = { x: 0, y: 0 };
//   zoomLevel.value = 1;
// }
function getCenterPointOfElement(el: DiagramElementData) {
  if (!el) return null;
  // TODO unsure if this edge check will work
  if (el instanceof DiagramEdgeData) {
    const fromPoint = getSocketLocationInfo("from", el)?.center;
    const toPoint = getSocketLocationInfo("to", el)?.center;
    if (!fromPoint || !toPoint) return;
    return pointAlongLinePct(fromPoint, toPoint, 0.5);
  } else if ("componentId" in el.def) {
    const comp =
      viewsStore.components[el.def.id] || viewsStore.groups[el.def.id]!;
    const position = { ...comp };
    position.y += position.height / 2;
    return position;
  }
}

function recenterOnElement(panTarget: DiagramElementData) {
  const centerOnPoint = getCenterPointOfElement(panTarget);
  if (centerOnPoint) {
    gridOrigin.value = centerOnPoint;
  }
}

const renameInputRef = ref<InstanceType<typeof VormInput>>();
const renameInputWrapperRef = ref();
const renameInputValue = ref("");
const renameElement = ref<DiagramGroupData | DiagramNodeData | undefined>();
const renameEndFunc = ref();

function fixRenameInputPosition() {
  if (renameElement.value) {
    const componentBox =
      viewsStore.components[renameElement.value.def.id] ||
      viewsStore.groups[renameElement.value.def.id];
    if (componentBox && renameInputWrapperRef.value) {
      const { x, y } = convertGridCoordsToContainerCoords(componentBox);
      const z = zoomLevel.value;

      if (!renameElement.value.def.isGroup) {
        // moving the input box for a Node
        const top = y + 3 * z;
        const left =
          z > 0.5
            ? x + 8 * z - (componentBox.width * z) / 2
            : x - (componentBox.width * z) / 2;
        const width =
          z > 0.5
            ? (componentBox.width - NODE_TITLE_HEADER_MARGIN_RIGHT) * z
            : componentBox.width * z;

        renameInputWrapperRef.value.style.top = `${top}px`;
        renameInputWrapperRef.value.style.left = `${left}px`;
        renameInputWrapperRef.value.style.width = `${width}px`;
      } else {
        // moving the input box for a Group
        const diffIcon =
          !renameElement.value.def.changeStatus ||
          renameElement.value.def.changeStatus === "unmodified";
        const width =
          z > 0.5
            ? (diffIcon
                ? componentBox.width - 2 - GROUP_HEADER_ICON_SIZE
                : componentBox.width - 8 - GROUP_HEADER_ICON_SIZE * 2) * z
            : componentBox.width * z;
        const top = y - 58 * z;
        const left =
          z > 0.5 ? x - width / 2 + (diffIcon ? 20 : 0) * z : x - width / 2;

        renameInputWrapperRef.value.style.top = `${top}px`;
        renameInputWrapperRef.value.style.left = `${left}px`;
        renameInputWrapperRef.value.style.width = `${width}px`;
      }
    }
  }
}

function renameOnDiagramByComponentId(componentId: ComponentId) {
  const component = componentsStore.allComponentsById[componentId];
  if (!component) return;
  const nodeRect =
    component instanceof DiagramNodeData
      ? viewsStore.components[component.def.id]
      : viewsStore.groups[component.def.id];
  if (!nodeRect) return;

  // TODO - for now, renaming from the event bus resets the zoom level
  gridOrigin.value = getRectCenter(nodeRect);
  setZoomLevel(1);
  renameOnDiagram(component, () => {});
}

function renameOnDiagram(
  el: DiagramNodeData | DiagramGroupData,
  endFunc: () => void,
) {
  const componentBox =
    viewsStore.components[el.def.id] || viewsStore.groups[el.def.id];

  if (componentBox && renameInputWrapperRef.value && renameInputRef.value) {
    renameElement.value = el;
    renameEndFunc.value = endFunc;
    renameInputValue.value = el.def.title;
    fixRenameInputPosition();
    renameInputRef.value.focus();
  }
}

function onRenameSubmit() {
  if (
    renameInputValue.value &&
    renameElement.value &&
    renameElement.value.def.title !== renameInputValue.value &&
    renameInputValue.value.length > 0
  ) {
    componentsStore.RENAME_COMPONENT(
      renameElement.value.def.id,
      renameInputValue.value,
    );
  }
  renameHide();
}

function renameHide() {
  if (renameInputWrapperRef.value && renameEndFunc.value) {
    renameInputWrapperRef.value.style.removeProperty("top");
    renameInputWrapperRef.value.style.removeProperty("left");
    renameInputWrapperRef.value.style.removeProperty("width");
    renameEndFunc.value(renameInputValue.value); // tells the component to show its title again!
    renameInputValue.value = "";
    renameElement.value = undefined;
  }
}

function onRenameKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    renameHide();
  }
}

const helpModalRef = ref();

onMounted(() => {
  componentsStore.copyingFrom = null;
  modelingEventBus.on("panToComponent", panToComponent);
  modelingEventBus.on("rename", renameOnDiagramByComponentId);
  modelingEventBus.on("setSelection", selectComponents);
});
onBeforeUnmount(() => {
  modelingEventBus.off("panToComponent", panToComponent);
  modelingEventBus.off("rename", renameOnDiagramByComponentId);
  modelingEventBus.off("setSelection", selectComponents);
});

const selectComponents = (components: ComponentId[]) => {
  viewsStore.setSelectedComponentId(components);
};

// this object gets provided to the children within the diagram that need it
const context: DiagramContext = {
  zoomLevel,
  setZoomLevel,
  drawEdgeState,
  moveElementsState,
  gridRect,
};
provide(DIAGRAM_CONTEXT_INJECTION_KEY, context);
</script>
