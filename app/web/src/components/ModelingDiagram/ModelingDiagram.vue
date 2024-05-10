Modeling diagram component * NOTE - uses a resize observer to react to size
changes, so this must be placed in a container that is sized explicitly has
overflow hidden */
<template>
  <div
    id="konva-container"
    ref="containerRef"
    :style="{ cursor }"
    class="absolute inset-0 overflow-hidden modeling-diagram"
  >
    <div
      v-if="fetchDiagramReqStatus.isFirstLoad"
      class="w-full h-full flex items-center bg-[rgba(0,0,0,.1)]"
    >
      <LoadingMessage message="Loading change set" />
    </div>
    <DiagramEmptyState v-else-if="componentsStore.diagramIsEmpty" />
    <div
      v-if="showDebugBar"
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
    <DiagramControls @open:help="helpModalRef.open()" />
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
      <DiagramGridBackground
        :gridMaxX="gridMaxX"
        :gridMaxY="gridMaxY"
        :gridMinX="gridMinX"
        :gridMinY="gridMinY"
        :zoomLevel="zoomLevel"
      />
      <v-layer>
        <DiagramGroup
          v-for="group in groups"
          :key="group.uniqueKey"
          :connectedEdges="connectedEdgesByElementKey[group.uniqueKey]"
          :group="group"
          :isHovered="elementIsHovered(group)"
          :isSelected="elementIsSelected(group)"
          :tempPosition="movedElementPositions[group.uniqueKey]"
          :tempSize="resizedElementSizes[group.uniqueKey]"
          @resize="onNodeLayoutOrLocationChange(group)"
        />
        <template v-if="edgeDisplayMode === 'EDGES_UNDER'">
          <DiagramEdge
            v-for="edge in edges"
            :key="edge.uniqueKey"
            :edge="edge"
            :fromPoint="getSocketLocationInfo(edge.fromSocketKey)?.center"
            :isHovered="elementIsHovered(edge)"
            :isSelected="elementIsSelected(edge)"
            :toPoint="getSocketLocationInfo(edge.toSocketKey)?.center"
          />
        </template>
        <DiagramNode
          v-for="node in nodes"
          :key="node.uniqueKey"
          :connectedEdges="connectedEdgesByElementKey[node.uniqueKey]"
          :isHovered="elementIsHovered(node)"
          :isSelected="elementIsSelected(node)"
          :node="node"
          :tempPosition="movedElementPositions[node.uniqueKey]"
          @resize="onNodeLayoutOrLocationChange(node)"
        />
        <DiagramCursor
          v-for="mouseCursor in presenceStore.diagramCursors"
          :key="mouseCursor.userId"
          :cursor="mouseCursor"
        />
        <template v-if="edgeDisplayMode === 'EDGES_OVER'">
          <DiagramEdge
            v-for="edge in edges"
            :key="edge.uniqueKey"
            :edge="edge"
            :fromPoint="getSocketLocationInfo(edge.fromSocketKey)?.center"
            :isHovered="elementIsHovered(edge)"
            :isSelected="elementIsSelected(edge)"
            :toPoint="getSocketLocationInfo(edge.toSocketKey)?.center"
          />
        </template>
        <DiagramGroupOverlay
          v-for="group in groups"
          :key="group.uniqueKey"
          :group="group"
          :tempPosition="movedElementPositions[group.uniqueKey]"
          :tempSize="resizedElementSizes[group.uniqueKey]"
          @resize="onNodeLayoutOrLocationChange(group)"
        />

        <!-- placeholders for new inserted elements still processing -->
        <template
          v-for="(
            pendingInsert, pendingInsertId
          ) in componentsStore.pendingInsertedComponents"
          :key="pendingInsertId"
        >
          <v-rect
            :config="{
            width: 160,
            height: 80,
            cornerRadius: CORNER_RADIUS,
            x: pendingInsert.position!.x - 80,
            y: pendingInsert.position!.y - 40,
            fill: 'rgba(0,0,0,.4)',
            strokeWidth: 1,
            stroke: SELECTION_COLOR,
          }"
          />
          <DiagramIcon
            :color="getToneColorHex('info')"
            :size="60"
            :x="pendingInsert.position!.x"
            :y="pendingInsert.position!.y"
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
          :fromPoint="getSocketLocationInfo(drawEdgeFromSocketKey)?.center"
          :toPoint="
            getSocketLocationInfo(drawEdgeToSocketKey)?.center || gridPointerPos
          "
        />
      </v-layer>
    </v-stage>
    <DiagramHelpModal ref="helpModalRef" />
  </div>
</template>

<script lang="ts">
type DiagramContext = {
  zoomLevel: Ref<number>;
  setZoomLevel: (newZoom: number) => void;
  edgeDisplayMode: Ref<EdgeDisplayMode>;
  toggleEdgeDisplayMode: () => void;
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
  reactive,
  watch,
  PropType,
  InjectionKey,
  ComputedRef,
  Ref,
  provide,
  inject,
} from "vue";
import { Stage as KonvaStage } from "konva/lib/Stage";
import * as _ from "lodash-es";
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d, IRect } from "konva/lib/types";
import tinycolor from "tinycolor2";
import { LoadingMessage, getToneColorHex } from "@si/vue-lib/design-system";
import { connectionAnnotationFitsReference } from "@si/ts-lib/src/connection-annotations";
import { windowListenerManager } from "@si/vue-lib";
import { useRoute } from "vue-router";
import { useCustomFontsLoaded } from "@/utils/useFontLoaded";
import DiagramGroup from "@/components/ModelingDiagram/DiagramGroup.vue";
import {
  EdgeId,
  useComponentsStore,
  FullComponent,
  ComponentPositions,
  ComponentConnection,
} from "@/store/components.store";
import DiagramGroupOverlay from "@/components/ModelingDiagram/DiagramGroupOverlay.vue";
import { DiagramCursorDef, usePresenceStore } from "@/store/presence.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { ComponentId } from "@/api/sdf/dal/component";
import { useAuthStore } from "@/store/auth.store";
import DiagramGridBackground from "./DiagramGridBackground.vue";
import {
  DiagramDrawEdgeState,
  DiagramNodeDef,
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
  EdgeDisplayMode,
  MoveElementsState,
  ComponentType,
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
  NODE_WIDTH,
  GROUP_INTERNAL_PADDING,
  GROUP_BOTTOM_INTERNAL_PADDING,
  GROUP_INNER_Y_BOUNDARY_OFFSET,
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
  shrinkRect,
} from "./utils/math";
import DiagramNewEdge from "./DiagramNewEdge.vue";
import { convertArrowKeyToDirection } from "./utils/keyboard";
import DiagramControls from "./DiagramControls.vue";
import DiagramHelpModal from "./DiagramHelpModal.vue";
import DiagramIcon from "./DiagramIcon.vue";
import DiagramEmptyState from "./DiagramEmptyState.vue";

const route = useRoute();

const changeSetsStore = useChangeSetsStore();
const realtimeStore = useRealtimeStore();
const authStore = useAuthStore();

// scroll pan multiplied by this and zoom level when panning
const ZOOM_PAN_FACTOR = 0.5;

const props = defineProps({
  cursors: {
    type: Array as PropType<DiagramCursorDef[]>,
    default: () => [],
  },
  // TODO: split this into controls for specific features rather than single toggle
  readOnly: { type: Boolean },
});

const emit = defineEmits<{
  (e: "right-click-element", elRightClickInfo: RightClickElementEvent): void;
}>();

const componentsStore = useComponentsStore();
const modelingEventBus = componentsStore.eventBus;

const edgeDisplayMode = ref<EdgeDisplayMode>("EDGES_OVER");

const toggleEdgeDisplayMode = () => {
  edgeDisplayMode.value =
    edgeDisplayMode.value === "EDGES_OVER" ? "EDGES_UNDER" : "EDGES_OVER";
};

const fetchDiagramReqStatus =
  componentsStore.getRequestStatus("FETCH_DIAGRAM_DATA");

const showDebugBar = false;

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
  const r = {
    x: gridMinX.value + v.x / zoomLevel.value,
    y: gridMinY.value + v.y / zoomLevel.value,
  };
  return r;
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

const pointerIsWithinGrid = computed(() => {
  if (!gridPointerPos.value) return false;
  const { x, y } = gridPointerPos.value;
  if (x < gridMinX.value || x > gridMaxX.value) return false;
  if (y < gridMinY.value || y > gridMaxY.value) return false;
  return true;
});

function onMouseWheel(e: KonvaEventObject<WheelEvent>) {
  // TODO check if target is the stage?
  e.evt.preventDefault();

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

  zoomLevel.value = newZoomLevel;
}

// not sure why but TS couldnt quite find the ResizeObserverCallback type...
type ResizeObserverCallback = ConstructorParameters<typeof ResizeObserver>[0];
const onResize: ResizeObserverCallback = (entries) => {
  entries.forEach((entry) => {
    if (!containerRef.value || entry.target !== containerRef.value) return;

    // using the resize observer lets us listen for resizes
    // and we'll assume location changes also happen with resizes for now

    // but resize observer wont help us get the element's position within the window
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

// fill both movedElementPositions and resizedElementSizes from data-loading
// and watch for new components entering the stage, fill them in here
watch(
  () => Object.keys(componentsStore.componentsById),
  () => {
    Object.values(componentsStore.componentsById).forEach((n) => {
      const elm = diagramDataFromNodeDef(n);

      // don't overwrite existing values (this causes elements to return to previous positions)
      const e: elementPositionAndSize = {
        uniqueKey: elm.uniqueKey,
        position: { x: n.position.x, y: n.position.y } as Vector2d,
      };
      if (n.isGroup && n.size?.height && n.size.width) {
        e.size = {
          width: n.size.width,
          height: n.size.height,
        } as Size2D;
      }
      if (!movedElementPositions[elm.uniqueKey]) {
        updateElementPositionAndSize({ elements: [e] }); // and don't save or broadcast
      }
    });
  },
);

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
          callback: ({ changeSetId, userPk, positions }) => {
            if (changeSetId !== changeSetsStore.selectedChangeSetId) return;
            if (userPk === authStore.userPk) return;

            const elements: elementPositionAndSize[] = [];
            for (const { componentId, position, size } of positions) {
              const gKey = DiagramGroupData.generateUniqueKey(componentId);
              const nKey = DiagramNodeData.generateUniqueKey(componentId);
              if (movedElementPositions[gKey]) {
                elements.push({
                  uniqueKey: gKey,
                  position,
                  size,
                } as elementPositionAndSize);
              } else if (movedElementPositions[nKey]) {
                elements.push({
                  uniqueKey: nKey,
                  position,
                });
              }
            }
            updateElementPositionAndSize({ elements });
          },
        },
      ],
    );
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  // this fires when you change the changeset from the drop down
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

  // console.log(e);

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
    componentsStore.selectedComponentIds.length
  ) {
    const component =
      componentsStore.componentsById[
        componentsStore.selectedComponentIds[0] ?? -1
      ];
    if (component) {
      // TODO: how to get copyingFrom
      window.localStorage.setItem(
        CLIPBOARD_LOCALSTORAGE_KEY.value,
        JSON.stringify({
          componentIds: componentsStore.selectedComponentIds,
          copyingFrom: component.position,
        }),
      );
    }
  } else if ((e.ctrlKey || e.metaKey) && e.key === "v") {
    const json = window.localStorage.getItem(CLIPBOARD_LOCALSTORAGE_KEY.value);
    if (json !== null && json !== "null") {
      try {
        const { componentIds, copyingFrom } = JSON.parse(json);
        componentsStore.selectedComponentIds = componentIds;
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
    if (insertElementActive.value) componentsStore.cancelInsert();
    componentsStore.copyingFrom = null;
    if (dragSelectActive.value) endDragSelect(false);
  }
  if (!props.readOnly && (e.key === "Delete" || e.key === "Backspace")) {
    modelingEventBus.emit("deleteSelection");
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
  else if (pasteElementsActive.value) triggerPasteElements();
  else handleMouseDownSelection();
}

function onMouseUp(e: MouseEvent) {
  // we dont care about right click
  if (e.button === 2) return;
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
  else if (pasteElementsActive.value && pointerIsWithinGrid.value)
    triggerPasteElements();
  else handleMouseUpSelection();
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
  } else if (lastMouseDownElement.value instanceof DiagramEdgeData) {
    // not sure what dragging an edge means... maybe nothing?
    /* eslint-disable-next-line no-console */
    // console.log("dragging edge ?");
  } else if (
    lastMouseDownElement.value instanceof DiagramNodeData ||
    lastMouseDownElement.value instanceof DiagramGroupData
  ) {
    if (lastMouseDownHoverMeta.value?.type === "resize") {
      beginResizeElement();
    } else if (
      lastMouseDownElement.value.def.changeStatus !== "deleted" &&
      lastMouseDownHoverMeta.value?.type === "socket"
    ) {
      // begin drawing edge
      beginDrawEdge(lastMouseDownHoverMeta.value.socket);
    } else {
      // begin moving selected nodes (and eventually other movable things like groups / annotations, etc...)
      beginDragElements();
    }
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
    return "pointer";
  }
  return "auto";
});

// HOVERING LOGIC + BEHAVIOUR //////////////////////////////////////////
const hoveredElementKey = computed(() => {
  if (componentsStore.hoveredComponentId) {
    return getDiagramElementKeyForComponentId(
      componentsStore.hoveredComponentId,
    );
  } else if (componentsStore.hoveredEdgeId) {
    return DiagramEdgeData.generateUniqueKey(componentsStore.hoveredEdgeId);
  }
  return undefined;
});
const hoveredElement = computed(() =>
  hoveredElementKey.value
    ? (allElementsByKey.value[hoveredElementKey.value] as
        | DiagramEdgeData
        | DiagramGroupData
        | DiagramNodeData)
    : undefined,
);

// same event and handler is used for both hovering nodes and sockets
// NOTE - we'll receive 2 events when hovering sockets, one for the node and one for the socket

// more detailed info about what inside an element is being hovered (like resize direction, socket, etc)
const hoveredElementMeta = computed(() => componentsStore.hoveredComponentMeta);

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
      componentsStore.componentsById[componentsStore.panTargetComponentId];
    if (!panToComponent) return;

    const key =
      panToComponent.componentType === ComponentType.Component
        ? DiagramNodeData.generateUniqueKey(panToComponent.id)
        : DiagramGroupData.generateUniqueKey(panToComponent.id);

    const el = getElementByKey(key);
    if (el) recenterOnElement(el);
    componentsStore.panTargetComponentId = null;
  },
);

// TODO: handle multiple components?
function panToComponent(payload: {
  componentId: ComponentId;
  center?: boolean;
}) {
  const key = getDiagramElementKeyForComponentId(payload.componentId);
  if (!key) return;
  const el = allElementsByKey.value[key];
  if (!el) return;

  const nodeRect = nodesLocationInfo[el.uniqueKey];
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
  if (componentsStore.selectedEdgeId) {
    return _.compact([
      getDiagramElementKeyForEdgeId(componentsStore.selectedEdgeId),
    ]);
  } else {
    return _.compact(
      _.map(
        componentsStore.selectedComponentIds,
        getDiagramElementKeyForComponentId,
      ),
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
    componentsStore.setSelectedComponentId(null);
    return;
  }

  const els = _.compact(_.map(_.castArray(toSelect), getElementByKey));

  if (els.length === 1 && els[0] instanceof DiagramEdgeData) {
    componentsStore.setSelectedEdgeId(els[0].def.id);
  } else {
    componentsStore.setSelectedComponentId(
      // TODO: remove this any...
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      _.map(els, (e) => (e.def as any).componentId),
    );
  }
}

// toggles selected items in the selection (used when shift clicking)
function toggleSelectedByKey(
  toToggle: DiagramElementUniqueKey | DiagramElementUniqueKey[],
) {
  const els = _.compact(_.map(_.castArray(toToggle), getElementByKey));
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const elIds = _.map(els, (el) => (el.def as any).componentId);
  // second true enables "toggle" mode
  componentsStore.setSelectedComponentId(elIds, { toggle: true });
}

function clearSelection() {
  componentsStore.setSelectedComponentId(null);
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
  if (
    hoveredElement.value instanceof DiagramNodeData ||
    hoveredElement.value instanceof DiagramGroupData
  ) {
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

  const selectedInBoxKeys: DiagramElementUniqueKey[] = [];
  _.each(nodesLocationInfo, (nodeRect, nodeKey) => {
    const inSelectionBox = checkRectanglesOverlap(
      pointsToRect(dragSelectStartPos.value!, dragSelectEndPos.value!),
      nodeRect,
    );
    if (inSelectionBox) selectedInBoxKeys.push(nodeKey);
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
      lastMouseDownElement.value instanceof DiagramGroupData ||
      lastMouseDownElement.value instanceof DiagramNodeData
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
}

function endDragSelect(doSelection = true) {
  dragSelectActive.value = false;
  if (doSelection) setSelectionByKey(dragSelectPreviewKeys.value);
}

/*
 * MOVING DIAGRAM ELEMENTS (nodes/groups/annotations/etc) ///////////////////////////////////////
 *
 * `movedElementPositions` and `resizedElementSizes` should only be SET via the updateElementPositionAndSize fn
 * You can read from those reactive dictionaries w/o issue
 */
const movedElementPositions = reactive<
  Record<DiagramElementUniqueKey, Vector2d>
>({});
const dragElementsActive = ref(false);
const currentSelectionMovableElements = computed(() => {
  // filter selection for nodes and groups
  const elements = _.filter(
    currentSelectionElements.value,
    (el) => el && "position" in el.def,
  ) as unknown as (DiagramNodeData | DiagramGroupData)[];

  // filter out children of other selected items, since moving a parent will already move the child
  const filteredElements = _.reject(elements, (el) => {
    const ancestors = el.def.ancestorIds;
    if (ancestors) {
      const parentKeys = ancestors.map(getDiagramElementKeyForComponentId);
      return _.intersection(currentSelectionKeys.value, parentKeys).length > 0;
    } else return false;
  });

  return filteredElements;
});

const draggedElementsPositionsPreDrag = ref<
  Record<DiagramElementUniqueKey, Vector2d | undefined>
>({});
const totalScrolledDuringDrag = ref<Vector2d>({ x: 0, y: 0 });

interface elementPositionAndSize {
  uniqueKey: DiagramElementUniqueKey;
  position?: Vector2d;
  size?: Size2D; // only frames have a size
}

interface updateElementPositionAndSizeArgs {
  elements: elementPositionAndSize[];
  writeToChangeSet?: boolean;
  broadcastToClients?: boolean;
}
function updateElementPositionAndSize(e: updateElementPositionAndSizeArgs) {
  /*
    Replaces most common uses of `sendMovedElementPosition`
    Nearly every instance of `send...` also mutated these two dictionaries, encapsulating that logic
    It will also call `send...`, optionally
  */
  _.forEach(e.elements, (e) => {
    if (e.position) {
      movedElementPositions[e.uniqueKey] = { ...e.position };
    }
    if (e.size) {
      resizedElementSizes[e.uniqueKey] = { ...e.size };
    }
  });

  // for convience
  if (e.writeToChangeSet || e.broadcastToClients) {
    sendMovedElementPosition({
      ...e,
      uniqueKeys: e.elements.map((e) => e.uniqueKey),
    });
  }
}

function sendMovedElementPosition(e: {
  // used to send the already existing elements position and size
  uniqueKeys: DiagramElementUniqueKey[];
  writeToChangeSet?: boolean;
  broadcastToClients?: boolean;
}) {
  if (!e.writeToChangeSet && !e.broadcastToClients) return;
  if (!e.uniqueKeys) return;

  const positions: ComponentPositions[] = [];
  for (const key of e.uniqueKeys) {
    const position = movedElementPositions[key];
    if (position) {
      position.x = Math.round(position.x);
      position.y = Math.round(position.y);
    }
    const size = resizedElementSizes[key];
    if (size) {
      size.width = Math.round(size.width);
      size.height = Math.round(size.height);
    }
    const componentId = DiagramNodeData.componentIdFromUniqueKey(
      DiagramGroupData.componentIdFromUniqueKey(key),
    );
    if (position && componentId) {
      positions.push({ componentId, position, size });
    }
  }

  if (positions.length > 0) {
    if (e.writeToChangeSet) {
      componentsStore.SET_COMPONENT_DIAGRAM_POSITION(positions);
    }

    if (
      e.broadcastToClients &&
      changeSetsStore.selectedChangeSetId &&
      authStore.userPk
    ) {
      realtimeStore.sendMessage({
        kind: "ComponentSetPosition",
        data: {
          positions,
          userPk: authStore.userPk,
          changeSetId: changeSetsStore.selectedChangeSetId,
        },
      });
    }
  }
}

function beginDragElements() {
  if (!lastMouseDownElement.value) return;
  dragElementsActive.value = true;

  totalScrolledDuringDrag.value = { x: 0, y: 0 };

  draggedElementsPositionsPreDrag.value = _.mapValues(
    allElementsByKey.value,
    (el) => movedElementPositions[el.uniqueKey] || _.get(el.def, "position"),
  );
}

function endDragElements() {
  dragElementsActive.value = false;
  // fire off final move event
  // note - we've already filtered out children that are selected along with their parents,
  // so we don't need to worry about accidentally moving them more than once
  const numSelected = currentSelectionMovableElements.value.length;
  const keys_to_save: string[] = [];
  const parentConnections: ComponentConnection[] = [];
  const childrenToDetach: ComponentId[] = [];
  _.each(currentSelectionMovableElements.value, (el) => {
    let fitWithinParent: null | {
      position: Vector2d;
      size: Size2D;
    } = null;

    if (!movedElementPositions[el.uniqueKey]) return;

    // dragging onto root - ie detach from all parents
    if (!cursorWithinGroupKey.value) {
      if (el.def.parentId) {
        childrenToDetach.push(el.def.componentId);
      }
    } else {
      const newParent = allElementsByKey.value[
        cursorWithinGroupKey.value
      ] as DiagramGroupData;

      if (el.def.parentId !== newParent?.def.componentId) {
        if (numSelected === 1 && newParent?.def.childIds?.length === 0) {
          if (newParent.def.size) {
            fitWithinParent = {
              position:
                movedElementPositions[newParent.uniqueKey] ??
                newParent.def.position,
              size:
                resizedElementSizes[newParent.uniqueKey] ?? newParent.def.size,
            };
          }
        }
        parentConnections.push({
          childId: el.def.id,
          parentId: newParent.def.id,
        });
      }
    }

    // only resize if my parent *currently* has zero children, and i have no children
    // and i'm only adding one component into the frame (e.g. we can't "grow" two children to max size)
    // otherwise let the human deal witht it
    keys_to_save.push(el.uniqueKey);
    if (numSelected === 1 && el.def.childIds?.length === 0 && fitWithinParent) {
      const [position, size] = fitChildInsideParentFrame(
        fitWithinParent.position,
        fitWithinParent.size,
      );
      updateElementPositionAndSize({
        elements: [
          {
            uniqueKey: el.uniqueKey,
            position,
            size,
          },
        ],
      });
    }

    if (el instanceof DiagramGroupData) {
      const childEls = allChildren(el);
      keys_to_save.push(..._.map(childEls, (childEl) => childEl.uniqueKey));
    }
  });

  // send one batch of all positions that are being moved
  sendMovedElementPosition({
    uniqueKeys: keys_to_save,
    writeToChangeSet: true,
  });

  if (parentConnections.length > 0)
    componentsStore.CONNECT_COMPONENT_TO_FRAME(parentConnections);

  if (childrenToDetach.length > 0)
    componentsStore.DETACH_COMPONENT(childrenToDetach);
}

let dragToEdgeScrollInterval: ReturnType<typeof setInterval> | undefined;

// TODO(victor) This can be optimized
// This needs to be recursive to find child components at all depths
const allChildren = (el: DiagramGroupData): DiagramElementData[] => {
  const children: DiagramElementData[] = [];
  _.map(el.def.childIds, (childId) => {
    const c = nodes.value.find((n) => n.def.id === childId);
    if (c) children.push(c);
    const g = groups.value.find((n) => n.def.id === childId);
    if (g) {
      children.push(g);
      children.push(...allChildren(g));
    }
  });
  return children;
};

function onDragElementsMove() {
  if (!containerPointerPos.value) return;
  if (!lastMouseDownContainerPointerPos.value) return;
  const delta: Vector2d = {
    x: Math.round(
      (containerPointerPos.value.x -
        lastMouseDownContainerPointerPos.value.x +
        totalScrolledDuringDrag.value.x) /
        zoomLevel.value,
    ),
    y: Math.round(
      (containerPointerPos.value.y -
        lastMouseDownContainerPointerPos.value.y +
        totalScrolledDuringDrag.value.y) /
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

  const elements: elementPositionAndSize[] = [];
  _.each(currentSelectionMovableElements.value, (el) => {
    if (!draggedElementsPositionsPreDrag.value?.[el.uniqueKey]) return;
    const newPosition = vectorAdd(
      draggedElementsPositionsPreDrag.value[el.uniqueKey]!,
      delta,
    );

    // if we are going to move the element within a new parent we may need to adjust
    // the position to stay inside of it
    if (parentOrCandidate) {
      const parentRect = nodesLocationInfo[parentOrCandidate.uniqueKey];
      const elRect = nodesLocationInfo[el.uniqueKey];
      if (!parentRect || !elRect) return;
      const movedElRect = {
        x: newPosition.x - elRect.width / 2,
        y: newPosition.y,
        width: elRect.width,
        height: elRect.height,
      };

      const parentRectWithBuffer = shrinkRect(parentRect, 20);

      if (!rectContainsAnother(parentRectWithBuffer, movedElRect)) {
        const adjust = getAdjustmentRectToContainAnother(
          parentRectWithBuffer,
          movedElRect,
        );
        newPosition.x -= adjust.x;
        newPosition.y -= adjust.y;
      }
    }

    if (el instanceof DiagramGroupData) {
      const includedGroups: DiagramNodeData[] & DiagramGroupData[] = [];
      const cycleCheck = new Set();
      const queue = [el];
      while (queue.length > 0) {
        cycleCheck.add(el.def.id);

        const parent = queue.shift();
        const x = _.filter(
          groups.value,
          (n) => n.def.parentId === parent?.def.id,
        );
        _.each(x, (childGroup) => {
          if (cycleCheck.has(childGroup.def.id)) return;
          queue.push(childGroup);
          includedGroups.push(childGroup);
        });
      }

      const childEls = allChildren(el);

      const actualParentDelta = vectorBetween(
        draggedElementsPositionsPreDrag.value[el.uniqueKey]!,
        newPosition,
      );

      _.each(childEls, (childEl) => {
        if (!draggedElementsPositionsPreDrag.value?.[childEl.uniqueKey]) return;

        const newChildPosition = vectorAdd(
          draggedElementsPositionsPreDrag.value[childEl.uniqueKey]!,
          actualParentDelta,
        );

        // track the position locally, so we don't need to rely on parent to store the temporary position
        elements.push({
          uniqueKey: childEl.uniqueKey,
          position: newChildPosition,
        });
      });
    }

    // track the position locally, so we don't need to rely on parent to store the temporary position
    elements.push({ uniqueKey: el.uniqueKey, position: newPosition });
  });
  updateElementPositionAndSize({
    elements,
    writeToChangeSet: false,
    broadcastToClients: true,
  });

  // check if dragging to the edge of the screen, which will trigger scrolling
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
  totalScrolledDuringDrag.value.x += deltaX;
  totalScrolledDuringDrag.value.y += deltaY;

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
  const positions = _.map(
    currentSelectionMovableElements.value,
    (el) => el.def.position,
  );
  const xPositions = _.map(positions, (p) => p.x);
  const yPositions = _.map(positions, (p) => p.y);
  if (direction === "up") alignedY = _.min(yPositions);
  else if (direction === "down") alignedY = _.max(yPositions);
  else if (direction === "left") alignedX = _.min(xPositions);
  else if (direction === "right") alignedX = _.max(xPositions);
  const elements: elementPositionAndSize[] = [];
  _.forEach(currentSelectionMovableElements.value, (el) => {
    const position = {
      x: alignedX === undefined ? el.def.position.x : alignedX,
      y: alignedY === undefined ? el.def.position.y : alignedY,
    } as Vector2d;
    elements.push({
      position,
      uniqueKey: el.uniqueKey,
    });
  });
  updateElementPositionAndSize({
    elements,
    writeToChangeSet: true,
  });
  // TODO: move viewport to show selection
}

type VoidFn = () => void;
let debouncedNudgeFn: _.DebouncedFunc<VoidFn> | null;
function nudgeSelection(direction: Direction, largeNudge: boolean) {
  if (!currentSelectionMovableElements.value.length) return;
  const nudgeSize = largeNudge ? 10 : 1;
  const nudgeVector: Vector2d = {
    left: { x: -1 * nudgeSize, y: 0 },
    right: { x: 1 * nudgeSize, y: 0 },
    up: { x: 0, y: -1 * nudgeSize },
    down: { x: 0, y: 1 * nudgeSize },
  }[direction];

  const elements = currentSelectionMovableElements.value.reduce<
    elementPositionAndSize[]
  >((elms, el) => {
    const e = recursivePositionCompute(el, nudgeVector);
    elms.push(...e);
    return elms;
  }, []);
  updateElementPositionAndSize({
    elements,
    broadcastToClients: true,
  });
  const uniqueKeys = elements.map((e) => e.uniqueKey);
  if (!debouncedNudgeFn) {
    debouncedNudgeFn = _.debounce(() => {
      sendMovedElementPosition({
        uniqueKeys,
        writeToChangeSet: true,
      });
      debouncedNudgeFn = null;
    }, 300);
  }
  debouncedNudgeFn();
}

const recursivePositionCompute = (
  el: DiagramGroupData | DiagramNodeData,
  nudgeVector: Vector2d,
): elementPositionAndSize[] => {
  const elements: elementPositionAndSize[] = [];
  const newPosition = vectorAdd(
    movedElementPositions[el.uniqueKey] || el.def.position,
    nudgeVector,
  );
  elements.push({ uniqueKey: el.uniqueKey, position: newPosition });

  const component = componentsStore.componentsById[el.def.componentId];

  _.each(component?.childIds, (id) => {
    if (
      !currentSelectionMovableElements.value.find(
        (e) => e.def.componentId === id,
      )
    ) {
      const key = getDiagramElementKeyForComponentId(id);
      const node = getElementByKey(key);

      if (node instanceof DiagramNodeData || node instanceof DiagramGroupData) {
        elements.push(...recursivePositionCompute(node, nudgeVector));
      }
    }
  });
  return elements;
};

// we calculate which group (if any) the cursor is within without using hover events
// which is useful when dragging elements in/out of groups
const cursorWithinGroupKey = computed(() => {
  // groups are sorted by depth so they render in the right order
  // so we search from the opposite direction to find deepest child first
  if (!gridPointerPos.value) return undefined;

  const withinGroup = _.findLast(groups.value, (group) => {
    // skip groups that are selected
    if (currentSelectionKeys.value.includes(group.uniqueKey)) return false;

    const frameRect = nodesLocationInfo[group.uniqueKey];
    if (!frameRect) return false;
    return rectContainsPoint(frameRect, gridPointerPos.value!);
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
const resizedElementSizes = reactive<Record<DiagramElementUniqueKey, Size2D>>(
  {},
);
const resizedElementSizesPreResize = reactive<
  Record<DiagramElementUniqueKey, Size2D>
>({});

const frameBoundingBoxes = ref<Record<string, IRect>>({});

// Calculate content bounding boxes for every group
watch(
  [resizedElementSizes, isMounted, movedElementPositions, stageRef],
  () => {
    if (!kStage) return;

    const boxDictionary: Record<string, IRect> = {};

    for (const group of groups.value) {
      const childIds = group.def.childIds;
      if (!childIds) continue;

      let top;
      let bottom;
      let left;
      let right;
      for (const childId of childIds) {
        const child = _.concat(groups.value, nodes.value).find(
          (c) => c.def.id === childId,
        );
        if (!child) continue;
        const elShape = kStage.findOne(`#${child.uniqueKey}--bg`);
        if (!elShape) continue;

        const position =
          movedElementPositions[child.uniqueKey] ?? child.def.position;
        let size = resizedElementSizes[child.uniqueKey] ?? child.def.size;
        if (!size) {
          size = { width: elShape.width(), height: elShape.height() };
        }

        const geometry = {
          ...position,
          ...size,
        };

        if (child instanceof DiagramGroupData) {
          geometry.y -= GROUP_INNER_Y_BOUNDARY_OFFSET;
          geometry.height += GROUP_INNER_Y_BOUNDARY_OFFSET;
        }

        if (!top || geometry.y < top) top = geometry.y;

        const thisLeft = geometry.x - geometry.width / 2;
        if (!left || thisLeft < left) left = thisLeft;

        const thisRight = geometry.x + geometry.width / 2;
        if (!right || thisRight > right) right = thisRight;

        const thisBottom = geometry.y + geometry.height;
        if (!bottom || thisBottom > bottom) bottom = thisBottom;
      }

      if (
        left === undefined ||
        right === undefined ||
        top === undefined ||
        bottom === undefined
      )
        continue;

      // TODO(Wendy) - Eventually we need to decide what happens if you add a Frame to another Frame that is smaller than it!
      boxDictionary[group.uniqueKey] = {
        x: left - GROUP_INTERNAL_PADDING,
        y: top - GROUP_INTERNAL_PADDING,
        width: right - left + GROUP_INTERNAL_PADDING * 2,
        height:
          bottom - top + GROUP_INTERNAL_PADDING + GROUP_BOTTOM_INTERNAL_PADDING,
      };
    }

    frameBoundingBoxes.value = boxDictionary;
  },
  { immediate: true },
);

function beginResizeElement() {
  if (!lastMouseDownElement.value) return;
  if (lastMouseDownHoverMeta.value?.type !== "resize") return;

  const node = lastMouseDownElement.value.def as DiagramNodeDef;

  if (!node.size) return;
  if (!(lastMouseDownElement.value instanceof DiagramGroupData)) return;

  resizeElement.value = lastMouseDownElement.value;
  resizeElementDirection.value = lastMouseDownHoverMeta.value.direction;

  const resizeTargetKey = lastMouseDownElement.value.uniqueKey;
  resizedElementSizesPreResize[resizeTargetKey] =
    resizedElementSizes[resizeTargetKey] || node.size;

  draggedElementsPositionsPreDrag.value[resizeTargetKey] =
    movedElementPositions[resizeTargetKey] || node.position;
}

function endResizeElement() {
  const el = resizeElement.value;
  if (!el) return;
  // currently only groups can be resized... this is mostly for TS
  if (!(el instanceof DiagramGroupData)) return;

  const size = resizedElementSizes[el.uniqueKey];
  const position = movedElementPositions[el.uniqueKey];
  if (!size || !position) {
    return;
  }
  updateElementPositionAndSize({
    elements: [{ position, size, uniqueKey: el.uniqueKey }],
    writeToChangeSet: true,
  });

  resizeElement.value = undefined;
}

const MIN_NODE_DIMENSION = NODE_WIDTH + 20 * 2;
function onResizeMove() {
  if (!resizeElement.value || !resizeElementDirection.value) return;
  const resizeTargetKey = resizeElement.value.uniqueKey;

  const node = resizeElement.value.def as DiagramNodeDef;

  if (!node.size) return;
  if (!containerPointerPos.value) return;
  if (!lastMouseDownContainerPointerPos.value) return;
  const sizeDelta: Vector2d = {
    x: Math.round(
      (containerPointerPos.value.x -
        lastMouseDownContainerPointerPos.value.x +
        totalScrolledDuringDrag.value.x) /
        zoomLevel.value,
    ),
    y: Math.round(
      (containerPointerPos.value.y -
        lastMouseDownContainerPointerPos.value.y +
        totalScrolledDuringDrag.value.y) /
        zoomLevel.value,
    ),
  };

  const positionDelta: Vector2d = {
    x: 0,
    y: 0,
  };

  const presentSize = resizedElementSizesPreResize[resizeTargetKey];
  const presentPosition =
    draggedElementsPositionsPreDrag.value[resizeTargetKey];

  if (!presentSize || !presentPosition) {
    return;
  }

  const rightBound = presentPosition.x + presentSize.width / 2;

  switch (resizeElementDirection.value) {
    case "bottom":
      {
        sizeDelta.x = 0;
        const minDelta = MIN_NODE_DIMENSION - presentSize.height;
        if (sizeDelta.y < minDelta) {
          sizeDelta.y = minDelta;
        }
      }
      break;
    case "top":
      {
        sizeDelta.x = 0;
        sizeDelta.y = -sizeDelta.y;
        const minDelta = MIN_NODE_DIMENSION - presentSize.height;
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
        const minDelta = MIN_NODE_DIMENSION - presentSize.width;
        if (sizeDelta.x < minDelta) {
          sizeDelta.x = minDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "right":
      {
        sizeDelta.y = 0;
        const minDelta = MIN_NODE_DIMENSION - presentSize.width;
        if (sizeDelta.x < minDelta) {
          sizeDelta.x = minDelta;
        }
        positionDelta.x = sizeDelta.x;
      }
      break;
    case "bottom-left":
      {
        const minYDelta = MIN_NODE_DIMENSION - presentSize.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }

        sizeDelta.x = -sizeDelta.x;
        const minXDelta = MIN_NODE_DIMENSION - presentSize.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "bottom-right":
      {
        const minYDelta = MIN_NODE_DIMENSION - presentSize.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }
        const minXDelta = MIN_NODE_DIMENSION - presentSize.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = sizeDelta.x;
      }
      break;
    case "top-left":
      {
        sizeDelta.y = -sizeDelta.y;
        const minYDelta = MIN_NODE_DIMENSION - presentSize.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }
        positionDelta.y = -sizeDelta.y;

        sizeDelta.x = -sizeDelta.x;
        const minXDelta = MIN_NODE_DIMENSION - presentSize.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "top-right":
      {
        sizeDelta.y = -sizeDelta.y;
        const minYDelta = MIN_NODE_DIMENSION - presentSize.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }
        positionDelta.y = -sizeDelta.y;

        const minXDelta = MIN_NODE_DIMENSION - presentSize.width;
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
    width: presentSize.width + sizeDelta.x,
    height: presentSize.height + sizeDelta.y,
  };

  // Get the correctly cached position for the element being resized
  const newNodePosition = {
    x: presentPosition.x + positionDelta.x / 2,
    y: presentPosition.y + positionDelta.y,
  };

  // Make sure the frame doesn't shrink to be smaller than it's children
  const contentsBox = frameBoundingBoxes.value[resizeTargetKey];

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
          presentPosition.y + presentSize.height - contentsBox.y;
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
  const parentId = node.parentId;

  if (parentId) {
    // Resized element with top-left corner xy coordinates instead of top-center
    const newNodeRect = {
      ...newNodePosition,
      ...newNodeSize,
      x: newNodePosition.x - newNodeSize.width / 2,
    };

    const parent = groups.value.find((g) => g.def.componentId === parentId);
    const parentShape = kStage.findOne(`#${parent?.uniqueKey}--bg`);
    if (parent && parentShape) {
      const parentPosition =
        movedElementPositions[parent.uniqueKey] ?? parent.def.position;

      const parentContentRect = {
        x: parentPosition.x - parentShape.width() / 2 + GROUP_INTERNAL_PADDING,
        y: parentPosition.y + GROUP_INTERNAL_PADDING,
        width: parentShape.width() - GROUP_INTERNAL_PADDING * 2,
        height:
          parentShape.height() -
          GROUP_INTERNAL_PADDING -
          GROUP_BOTTOM_INTERNAL_PADDING,
      };

      // Top Collision
      if (parentContentRect.y > newNodeRect.y - GROUP_INNER_Y_BOUNDARY_OFFSET) {
        newNodeRect.y = parentContentRect.y + GROUP_INNER_Y_BOUNDARY_OFFSET;
        newNodeRect.height =
          presentPosition.y +
          presentSize.height -
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
    }

    newNodePosition.x = newNodeRect.x + newNodeRect.width / 2;
    newNodePosition.y = newNodeRect.y;
    newNodeSize.width = newNodeRect.width;
    newNodeSize.height = newNodeRect.height;
  }

  updateElementPositionAndSize({
    elements: [
      {
        uniqueKey: resizeElement.value.uniqueKey,
        position: newNodePosition,
        size: newNodeSize,
      },
    ],
    broadcastToClients: true,
  });
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
  const fromSocket = drawEdgeFromSocket.value;
  if (!drawEdgeActive.value || !fromSocket) return [];

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
  const fromSocket = drawEdgeFromSocket.value;
  const toSocket = drawEdgeToSocket.value;
  if (!drawEdgeActive.value || !fromSocket) return [];

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
      const socketLocation = socketsLocationInfo[socketKey];
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

  await componentsStore.CREATE_COMPONENT_CONNECTION(
    {
      componentId: fromComponentId,
      socketId: fromSocketId,
    },
    {
      componentId: toComponentId,
      socketId: toSocketId,
    },
  );
}

const pasteElementsActive = computed(() => {
  return (
    componentsStore.copyingFrom &&
    componentsStore.selectedComponentIds.length > 0
  );
});

async function triggerPasteElements() {
  if (!pasteElementsActive.value)
    throw new Error("paste element mode must be active");
  if (!gridPointerPos.value)
    throw new Error("Cursor must be in grid to paste element");
  if (!componentsStore.copyingFrom)
    throw new Error("Copy cursor must be in grid to paste element");

  const newParentNodeId =
    allElementsByKey.value[cursorWithinGroupKey.value ?? "-1"]?.def.id;
  const copyingFrom = componentsStore.copyingFrom;
  componentsStore.copyingFrom = null;

  await componentsStore.PASTE_COMPONENTS(
    componentsStore.selectedComponentIds,
    {
      x: gridPointerPos.value.x - copyingFrom.x,
      y: gridPointerPos.value.y - copyingFrom.y,
    },
    gridPointerPos.value,
    newParentNodeId,
  );
}

// ELEMENT ADDITION
const insertElementActive = computed(
  () => !!componentsStore.selectedInsertSchemaId,
);

function fitChildInsideParentFrame(
  position: Vector2d,
  size: Size2D,
): [Vector2d, Size2D] {
  // position the component within its parent cleanly
  const HEADER_SIZE = 60; // The height of the compoennt header bar; TODO find a better way to detect this
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

  // enforce minimums
  createAtSize.width = Math.max(createAtSize.width, MIN_NODE_DIMENSION);
  createAtSize.height = Math.max(createAtSize.height, MIN_NODE_DIMENSION);

  return [createAtPosition, createAtSize];
}

async function triggerInsertElement() {
  if (!insertElementActive.value)
    throw new Error("insert element mode must be active");
  if (!gridPointerPos.value)
    throw new Error("Cursor must be in grid to insert element");

  if (!componentsStore.selectedInsertSchemaId)
    throw new Error("missing insert selection metadata");

  const schemaId = componentsStore.selectedInsertSchemaId;
  componentsStore.selectedInsertSchemaId = null;

  const parentGroupId: string | undefined = cursorWithinGroupKey.value?.replace(
    "g-",
    "",
  );

  let parentId;
  let createAtSize: Size2D | undefined;
  let createAtPosition = gridPointerPos.value;

  if (parentGroupId) {
    const parentComponent = componentsStore.componentsById[parentGroupId];
    if (
      parentComponent &&
      (parentComponent.componentType !== ComponentType.AggregationFrame ||
        schemaId === parentComponent.schemaId)
    ) {
      parentId = parentGroupId;
    }

    if (parentComponent) {
      if (parentComponent.childIds.length > 0) {
        // when there are already children we can't be as smart
        // leave position as the cursor
        // backend default is 500 x 500, just make it smaller since there are other children
        createAtSize = { width: 250, height: 250 };
      } else if (parentComponent.position && parentComponent.size) {
        [createAtPosition, createAtSize] = fitChildInsideParentFrame(
          parentComponent.position,
          parentComponent.size,
        );
      }
    }
  }

  // as this stands, the client will send a width/height for non-frames, that the API endpoint ignores
  // TODO: is there is a good way to determine whether this schemaID is a frame?
  componentsStore.CREATE_COMPONENT(
    schemaId,
    createAtPosition,
    parentId,
    createAtSize,
  );
}

// LAYOUT REGISTRY + HELPERS ///////////////////////////////////////////////////////////
type SocketLocationInfo = { center: Vector2d };
const nodesLocationInfo = reactive<Record<string, IRect>>({});
const socketsLocationInfo = reactive<Record<string, SocketLocationInfo>>({});

function getSocketLocationInfo(socketKey?: DiagramElementUniqueKey) {
  if (!socketKey) return undefined;
  return socketsLocationInfo[socketKey];
}

function onNodeLayoutOrLocationChange(el: DiagramNodeData | DiagramGroupData) {
  // record node location/dimensions (used when drawing selection box)
  // we find the background shape, because the parent group has no dimensions
  const nodeBgShape = kStage.findOne(`#${el.uniqueKey}--bg`);
  nodesLocationInfo[el.uniqueKey] = {
    ...nodeBgShape.getAbsolutePosition(kStage),
    width: nodeBgShape.width(),
    height: nodeBgShape.height(),
  };

  if ("sockets" in el) {
    // record new socket locations (used to render edges)
    _.each(el.sockets, (socket) => {
      const socketShape = kStage.findOne(`#${socket.uniqueKey}`);
      // This ensures that the diagram won't try to create edges to/from hidden sockets
      if (!socketShape) return;
      socketsLocationInfo[socket.uniqueKey] = {
        center: socketShape.getAbsolutePosition(kStage),
      };
    });
  }
}

// DIAGRAM CONTENTS HELPERS //////////////////////////////////////////////////

// TODO: DiagramNodeDef and FullComponent are almost identical. We don't need both.
function diagramDataFromNodeDef(
  nodeDef: DiagramNodeDef | FullComponent,
): DiagramNodeData | DiagramGroupData {
  const Cls =
    nodeDef.componentType === ComponentType.Component
      ? DiagramNodeData
      : DiagramGroupData;
  return new Cls(nodeDef as DiagramNodeDef);
}

const nodes = computed(() =>
  _.map(
    _.filter(
      componentsStore.diagramNodes,
      (n) => n.componentType === ComponentType.Component,
    ),
    (nodeDef) => new DiagramNodeData(nodeDef),
  ),
);
const groups = computed(() => {
  const allGroups = _.map(
    _.filter(
      componentsStore.diagramNodes,
      (n) => n.componentType !== ComponentType.Component,
    ),
    (groupDef) => {
      const g = new DiagramGroupData(groupDef);
      return g;
    },
  );
  const orderedGroups = _.orderBy(allGroups, (g) => {
    // order by "depth" in frames
    let zIndex = g.def.ancestorIds?.length || 0;

    // if being dragged (or ancestor being dragged), bump up to front, but maintain order within that frame
    if (dragElementsActive.value) {
      if (
        _.intersection(
          [g.def.componentId, ...(g.def.ancestorIds || [])],
          componentsStore.selectedComponentIds,
        ).length
      ) {
        zIndex += 1000;
      }
    }
    return zIndex;
  });

  return orderedGroups;
});
const sockets = computed(() =>
  _.compact(_.flatMap(_.concat(nodes.value, groups.value), (i) => i.sockets)),
);
const edges = computed(() =>
  _.map(
    componentsStore.diagramEdges,
    (edgeDef) => new DiagramEdgeData(edgeDef),
  ),
);

// quick ways to look up specific element data from a unique key
// const nodesByKey = computed(() => _.keyBy(nodes.value, (e) => e.uniqueKey));
// const groupsByKey = computed(() => _.keyBy(groups.value, (e) => e.uniqueKey));
// const socketsByKey = computed(() => _.keyBy(sockets.value, (e) => e.uniqueKey));
const edgesByKey = computed(() => _.keyBy(edges.value, (e) => e.uniqueKey));
const allElementsByKey = computed(() =>
  _.keyBy(
    [...nodes.value, ...groups.value, ...sockets.value, ...edges.value],
    (e) => e.uniqueKey,
  ),
);

function getElementByKey(key?: DiagramElementUniqueKey) {
  return key ? allElementsByKey.value[key] : undefined;
}

function getDiagramElementKeyForComponentId(
  componentId?: ComponentId | null,
): string | undefined {
  if (!componentId) return;
  const component = componentsStore.componentsById[componentId];
  if (component) {
    if (component.isGroup) {
      return DiagramGroupData.generateUniqueKey(component.id);
    }
    return DiagramNodeData.generateUniqueKey(component.id);
  }
}

function getDiagramElementKeyForEdgeId(edgeId?: EdgeId | null) {
  if (!edgeId) return;
  return DiagramEdgeData.generateUniqueKey(edgeId);
}

const connectedEdgesByElementKey = computed(() => {
  const lookup: Record<DiagramElementUniqueKey, DiagramEdgeData[]> = {};
  _.each(edgesByKey.value, (edge) => {
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
  if (el instanceof DiagramEdgeData) {
    // TODO: this logic should live on DiagramEdge class
    const fromPoint = getSocketLocationInfo(el.fromSocketKey)?.center;
    const toPoint = getSocketLocationInfo(el.toSocketKey)?.center;
    if (!fromPoint || !toPoint) return;
    return pointAlongLinePct(fromPoint, toPoint, 0.5);
  } else if (el instanceof DiagramNodeData || el instanceof DiagramGroupData) {
    // TODO: probably want nodes/groups to be able to return their correct center point
    const position = _.clone(
      movedElementPositions[el.uniqueKey] || el.def.position,
    );
    if (el.def.size) {
      position.y += el.def.size.height / 2;
    }
    return position;
  }
}

function recenterOnElement(panTarget: DiagramElementData) {
  const centerOnPoint = getCenterPointOfElement(panTarget);
  if (centerOnPoint) {
    gridOrigin.value = centerOnPoint;
  }
}

const helpModalRef = ref();

onMounted(() => {
  componentsStore.copyingFrom = null;
  componentsStore.eventBus.on("panToComponent", panToComponent);
});
onBeforeUnmount(() => {
  componentsStore.eventBus.off("panToComponent", panToComponent);
});

// this object gets provided to the children within the diagram that need it
const context: DiagramContext = {
  zoomLevel,
  setZoomLevel,
  edgeDisplayMode,
  toggleEdgeDisplayMode,
  drawEdgeState,
  moveElementsState,
  gridRect,
};
provide(DIAGRAM_CONTEXT_INJECTION_KEY, context);
</script>
