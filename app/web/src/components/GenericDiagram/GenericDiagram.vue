Generic diagram component * NOTE - uses a resize observer to react to size
changes, so this must be placed in a container that is sized explicitly has
overflow hidden */
<template>
  <div
    ref="containerRef"
    class="absolute inset-0 overflow-hidden"
    :style="{ cursor }"
  >
    <DiagramEmptyState v-if="props.nodes.length === 0" />
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
    <DiagramControls
      :zoomLevel="zoomLevel"
      @update:zoom="setZoom"
      @open:help="helpModalRef.open()"
    />
    <v-stage
      v-if="customFontsLoaded && containerWidth > 0 && containerHeight > 0"
      ref="stageRef"
      :config="{
        width: containerWidth,
        height: containerHeight,
        scale: { x: zoomLevel, y: zoomLevel },
        offset: { x: gridMinX, y: gridMinY },
      }"
      @mousedown="onMouseDown"
      @click.right="onRightClick"
    >
      <DiagramGridBackground
        :gridMinX="gridMinX"
        :gridMaxX="gridMaxX"
        :gridMinY="gridMinY"
        :gridMaxY="gridMaxY"
        :zoomLevel="zoomLevel"
      />
      <v-layer>
        <DiagramGroup
          v-for="group in groups"
          :key="group.uniqueKey"
          :group="group"
          :tempPosition="movedElementPositions[group.uniqueKey]"
          :tempSize="resizedElementSizes[group.uniqueKey]"
          :connectedEdges="connectedEdgesByElementKey[group.uniqueKey]"
          :drawEdgeState="drawEdgeState"
          :isHovered="elementIsHovered(group)"
          :isSelected="elementIsSelected(group)"
          @hover:start="(meta) => onElementHoverStart(group, meta)"
          @hover:end="onElementHoverEnd(group)"
          @resize="onNodeLayoutOrLocationChange(group)"
        />
        <DiagramNode
          v-for="node in nodes"
          :key="node.uniqueKey"
          :node="node"
          :tempPosition="movedElementPositions[node.uniqueKey]"
          :connectedEdges="connectedEdgesByElementKey[node.uniqueKey]"
          :drawEdgeState="drawEdgeState"
          :isHovered="elementIsHovered(node)"
          :isSelected="elementIsSelected(node)"
          :deleteIcon="deleteIcon"
          @hover:start="(meta) => onElementHoverStart(node, meta)"
          @hover:end="(meta) => onElementHoverEnd(node)"
          @resize="onNodeLayoutOrLocationChange(node)"
        />
        <DiagramEdge
          v-for="edge in edges"
          :key="edge.uniqueKey"
          :edge="edge"
          :fromPoint="getSocketLocationInfo(edge.fromSocketKey)?.center"
          :toPoint="getSocketLocationInfo(edge.toSocketKey)?.center"
          :isHovered="elementIsHovered(edge)"
          :isSelected="elementIsSelected(edge)"
          @hover:start="onElementHoverStart(edge)"
          @hover:end="onElementHoverEnd(edge)"
        />
        <DiagramGroupOverlay
          v-for="group in groups"
          :key="group.uniqueKey"
          :group="group"
          :tempPosition="movedElementPositions[group.uniqueKey]"
          :tempSize="resizedElementSizes[group.uniqueKey]"
          :deleteIcon="deleteIcon"
          @resize="onNodeLayoutOrLocationChange(group)"
        />

        <!-- placeholders for new inserted elements still processing -->
        <template
          v-for="(pendingInsert, pendingInsertId) in pendingInsertedElements"
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
            icon="loader"
            :color="diagramConfig?.toneColors?.['info'] || '#AAA'"
            :size="60"
            :x="pendingInsert.position!.x"
            :y="pendingInsert.position!.y"
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
} from "vue";
import { Stage as KonvaStage } from "konva/lib/Stage";
import * as _ from "lodash-es";
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d, IRect } from "konva/lib/types";
import tinycolor from "tinycolor2";
import { IconNames } from "@si/vue-lib/design-system";
import { useCustomFontsLoaded } from "@/utils/useFontLoaded";
import DiagramGroup from "@/components/GenericDiagram/DiagramGroup.vue";
import { useComponentsStore } from "@/store/components.store";
import DiagramGroupOverlay from "@/components/GenericDiagram/DiagramGroupOverlay.vue";
import DiagramGridBackground from "./DiagramGridBackground.vue";
import {
  DeleteElementsEvent,
  DiagramConfig,
  DiagramDrawEdgeState,
  DiagramEdgeDef,
  DiagramNodeDef,
  DrawEdgeEvent,
  MoveElementEvent,
  Direction,
  PendingInsertedElement,
  DiagramElementTypes,
  InsertElementEvent,
  RightClickElementEvent,
  DiagramNodeData,
  DiagramGroupData,
  DiagramEdgeData,
  DiagramSocketData,
  DiagramElementData,
  DiagramElementUniqueKey,
  SelectElementEvent,
  GroupEvent,
  Size2D,
  ResizeElementEvent,
  HoverElementEvent,
  SideAndCornerIdentifiers,
  ElementHoverMeta,
} from "./diagram_types";
import DiagramNode from "./DiagramNode.vue";
import DiagramEdge from "./DiagramEdge.vue";
import {
  useDiagramConfigProvider,
  useZoomLevelProvider,
} from "./utils/use-diagram-context-provider";
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
  GROUP_HEADER_ICON_SIZE,
  GROUP_HEADER_BOTTOM_MARGIN,
  GROUP_BOTTOM_INTERNAL_PADDING,
} from "./diagram_constants";
import {
  vectorDistance,
  vectorAdd,
  checkRectanglesOverlap,
  rectContainsAnother,
  vectorBetween,
} from "./utils/math";
import DiagramNewEdge from "./DiagramNewEdge.vue";
import { convertArrowKeyToDirection } from "./utils/keyboard";
import DiagramControls from "./DiagramControls.vue";
import DiagramHelpModal from "./DiagramHelpModal.vue";
import { baseConfig } from "./diagram_base_config";
import DiagramIcon from "./DiagramIcon.vue";
import DiagramEmptyState from "./DiagramEmptyState.vue";

// zoom config - zoom value of 1 is 100% zoom level
const ZOOM_SCROLL_FACTOR = 0.001; // scroll delta multiplied by this while zooming
// scroll pan multiplied by this and zoom level when panning
const ZOOM_PAN_FACTOR = 0.5;

const props = defineProps({
  customConfig: {
    type: Object as PropType<DiagramConfig>,
    default: () => ({}),
  },
  nodes: {
    type: Array as PropType<DiagramNodeDef[]>,
    default: () => [],
  },
  edges: {
    type: Array as PropType<DiagramEdgeDef[]>,
    default: () => [],
  },
  // TODO: split this into controls for specific features rather than single toggle
  readOnly: { type: Boolean },
  deleteIcon: { type: String as PropType<IconNames>, default: "x" },

  controlsDisabled: { type: Boolean },
});

const emit = defineEmits<{
  (e: "update:zoom", newZoom: number): void;
  (e: "update:selection", newSelection: SelectElementEvent): void;
  (e: "move-element", nodeMoveInfo: MoveElementEvent): void;
  (e: "hover-element", hoverInfo: HoverElementEvent): void;
  (e: "resize-element", nodeResizeInfo: ResizeElementEvent): void;
  (e: "delete-elements", deleteInfo: DeleteElementsEvent): void;
  (e: "insert-element", insertInfo: InsertElementEvent): void;
  (e: "draw-edge", drawEdgeInfo: DrawEdgeEvent): void;
  (e: "right-click-element", elRightClickInfo: RightClickElementEvent): void;
  (e: "group-elements", groupEvent: GroupEvent): void;
}>();

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
function setZoom(newZoomLevel: number) {
  if (newZoomLevel < MIN_ZOOM) zoomLevel.value = MIN_ZOOM;
  else if (newZoomLevel > MAX_ZOOM) zoomLevel.value = MAX_ZOOM;
  else zoomLevel.value = newZoomLevel;
}
watch(zoomLevel, () => {
  emit("update:zoom", zoomLevel.value);
});

// dimensions of our 2d grid space, all coordinates of things in the diagram are relative to this
const gridWidth = computed(() => containerWidth.value / zoomLevel.value);
const gridHeight = computed(() => containerHeight.value / zoomLevel.value);
// min/max values of the visible region of the diagram
const gridMinX = computed(() => gridOrigin.value.x - gridWidth.value / 2);
const gridMaxX = computed(() => gridOrigin.value.x + gridWidth.value / 2);
const gridMinY = computed(() => gridOrigin.value.y - gridHeight.value / 2);
const gridMaxY = computed(() => gridOrigin.value.y + gridHeight.value / 2);

function convertContainerCoordsToGridCoords(v: Vector2d): Vector2d {
  return {
    x: gridMinX.value + v.x / zoomLevel.value,
    y: gridMinY.value + v.y / zoomLevel.value,
  };
}

/** pointer position in frame of reference of container */
const containerPointerPos = ref<Vector2d>();
/** pointer position in frame of reference of grid  */
const gridPointerPos = computed(() => {
  if (!containerPointerPos.value) return undefined;
  const converted = convertContainerCoordsToGridCoords(
    containerPointerPos.value,
  );
  converted.x = Math.round(converted.x);
  converted.y = Math.round(converted.y);
  return converted;
});
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
  if (props.controlsDisabled) return;

  // if CMD key, treat wheel as zoom, otherwise pan
  if (e.evt.metaKey) {
    // e.evt.metaKey
    // zoom
    let newZoomLevel = zoomLevel.value - e.evt.deltaY * ZOOM_SCROLL_FACTOR;
    if (newZoomLevel < MIN_ZOOM) newZoomLevel = MIN_ZOOM;
    if (newZoomLevel > MAX_ZOOM) newZoomLevel = MAX_ZOOM;

    // need to move origin to zoom centered on pointer position
    if (containerPointerPos.value && gridPointerPos.value) {
      // this a little confusing, but we're recreating the same calculations as above, but but at the new zoom level
      // so we know where the pointer _would_ move and then offset the pointer position stays constant
      const newGridWidth = containerWidth.value / newZoomLevel;
      const newMinX = gridOrigin.value.x - newGridWidth / 2;
      const newGridHeight = containerHeight.value / newZoomLevel;
      const newMinY = gridOrigin.value.y - newGridHeight / 2;
      const pointerXAtNewZoom =
        newMinX + containerPointerPos.value.x / newZoomLevel;
      const pointerYAtNewZoom =
        newMinY + containerPointerPos.value.y / newZoomLevel;

      gridOrigin.value = {
        x: gridOrigin.value.x - (pointerXAtNewZoom - gridPointerPos.value.x),
        y: gridOrigin.value.y - (pointerYAtNewZoom - gridPointerPos.value.y),
      };
    }
    zoomLevel.value = newZoomLevel;
  } else {
    // pan
    const panFactor = zoomLevel.value * ZOOM_PAN_FACTOR;
    gridOrigin.value = {
      x: gridOrigin.value.x + e.evt.deltaX * panFactor,
      y: gridOrigin.value.y + e.evt.deltaY * panFactor,
    };
  }
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

// this is all a little ugly, but basically we are waiting until custom fonts are loaded to initialize and display the canvas
// or otherwise spacing gets messed up and we'd have to tell everything to rerender/recalculate when the fonts did get loaded
const isMounted = ref(false);
onMounted(() => {
  resizeObserver.observe(containerRef.value!);
  isMounted.value = true;
});

watch([customFontsLoaded, () => isMounted.value, () => stageRef.value], () => {
  if (!isMounted.value || !customFontsLoaded.value || !stageRef.value) return;
  onMountedAndReady();
});

function onMountedAndReady() {
  kStage = stageRef.value.getNode();
  kStage.on("wheel", onMouseWheel);
  // attach to window so we have coords even when mouse is outside bounds or on other elements
  // NOTE - mousedown is attached on the konva stage component above, since we only care about starting clicks within the diagram
  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);
}

onBeforeUnmount(() => {
  kStage?.off("wheel", onMouseWheel);
  window.removeEventListener("mousemove", onMouseMove);
  window.removeEventListener("mouseup", onMouseUp);
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("keyup", onKeyUp);
  resizeObserver.unobserve(containerRef.value!);
});

// global keyboard and mouse handlers, which reroute to the correct functionality

const spaceKeyIsDown = ref(false);
const shiftKeyIsDown = ref(false);

function onKeyDown(e: KeyboardEvent) {
  if (props.controlsDisabled) return;

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
    setZoom(zoomLevel.value + 0.1);
  }
  if (e.key === "-" || e.key === "_") {
    setZoom(zoomLevel.value - 0.1);
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
    if (insertElementActive.value) endInsertElement();
    if (dragSelectActive.value) endDragSelect(false);
  }
  if (!props.readOnly && (e.key === "Delete" || e.key === "Backspace")) {
    deleteSelected();
  }
}
function onKeyUp(e: KeyboardEvent) {
  if (e.key === " ") spaceKeyIsDown.value = false;
  if (e.key === "Shift") shiftKeyIsDown.value = false;
}

const mouseIsDown = ref(false);
const dragThresholdBroken = ref(false);
const lastMouseDownEvent = ref<MouseEvent>();
const lastMouseDownContainerPointerPos = ref<Vector2d>();
const lastMouseDownElementKey = ref<DiagramElementUniqueKey>();
const lastMouseDownHoverMeta = ref<ElementHoverMeta>();
const lastMouseDownElement = computed(() =>
  lastMouseDownElementKey.value
    ? allElementsByKey.value[lastMouseDownElementKey.value]
    : undefined,
);
function onMouseDown(ke: KonvaEventObject<MouseEvent>) {
  if (props.controlsDisabled) return;
  // not sure why, but this is being called twice, once with the konva event, and once with the bare event
  // so we ignore the bare event
  if (!ke.evt) return;
  const e = ke.evt;
  // we only care here about left click - might change this later...
  if (e.button !== 0) return;
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
  if (dragToPanArmed.value) beginDragToPan();
  else if (insertElementActive.value) triggerInsertElement();
  else handleMouseDownSelection();
}
function onMouseUp(e: MouseEvent) {
  if (props.controlsDisabled) return;
  // we only care here about left click - might change this later...
  if (e.button !== 0) return;
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
  else handleMouseUpSelection();
}
function onMouseMove(e: MouseEvent) {
  if (props.controlsDisabled) return;
  // update pointer location relative to container, which is used throughout
  containerPointerPos.value = {
    x: e.clientX - containerViewportX.value,
    y: e.clientY - containerViewportY.value,
  };

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
  if (props.controlsDisabled) return;
  const e = ke.evt;
  e.preventDefault(); // do not show browser right click menu
  if (!hoveredElement.value) return;

  if (!currentSelectionElements.value.includes(hoveredElement.value)) {
    setSelectionByKey(hoveredElement.value.uniqueKey);
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
  if (!lastMouseDownElement.value) {
    // begin drag to multi-select
    beginDragSelect();
  } else if (props.readOnly) {
    // TODO: add controls for each of these modes...
    return;
  } else if (lastMouseDownElement.value instanceof DiagramEdgeData) {
    // not sure what dragging an edge means... maybe nothing?
    /* eslint-disable-next-line no-console */
    console.log("dragging edge ?");
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

// Mode and cursor
const cursor = computed(() => {
  if (dragToPanActive.value) return "grabbing";
  if (dragToPanArmed.value) return "grab";
  if (dragSelectActive.value) return "crosshair";

  if (
    !props.readOnly &&
    hoveredElementMeta.value?.type === "socket" &&
    hoveredElement.value?.def.changeStatus !== "deleted"
  ) {
    return "cell";
  }
  if (drawEdgeActive.value) return "cell";
  if (dragElementsActive.value) return "move";
  if (insertElementActive.value) return "copy"; // not sure about this...
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
  if (hoveredElement.value) {
    return "pointer";
  }
  return "auto";
});

// hovering behaviour
const hoveredElementKey = ref<string>();
const hoveredElement = computed(() =>
  hoveredElementKey.value
    ? (allElementsByKey.value[hoveredElementKey.value] as
        | DiagramEdgeData
        | DiagramGroupData
        | DiagramNodeData)
    : undefined,
);

const hoveredElementMeta = ref<ElementHoverMeta>();

function setHoveredByKey(newHoverElementKey?: DiagramElementUniqueKey) {
  hoveredElementKey.value = newHoverElementKey;
}

// same event and handler is used for both hovering nodes and sockets
// NOTE - we'll receive 2 events when hovering sockets, one for the node and one for the socket

function onElementHoverStart(el: DiagramElementData, meta?: ElementHoverMeta) {
  hoveredElementKey.value = el.uniqueKey;
  hoveredElementMeta.value = meta;
  emit("hover-element", { element: el });
}
function onElementHoverEnd(_el: DiagramElementData) {
  hoveredElementKey.value = undefined;
  hoveredElementMeta.value = undefined;
  emit("hover-element", { element: null });
}

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
const panTarget = computed((_) => useComponentsStore().panTargetComponentId);
watch(panTarget, () => {
  if (!panTarget.value) return;

  const node = props.nodes.find((n) => n.componentId === panTarget.value);
  if (!node) return;

  const key =
    node.nodeType === "component"
      ? DiagramNodeData.generateUniqueKey(node.id)
      : DiagramGroupData.generateUniqueKey(node.id);

  const position = movedElementPositions[key] ?? _.clone(node.position);
  if (node.nodeType !== "component" && node.size) {
    position.y = node.position.y + node.size.height / 2;
  }

  gridOrigin.value = position;

  useComponentsStore().panTargetComponentId = null;
});

// ELEMENT SELECTION /////////////////////////////////////////////////////////////////////////////////
const _rawSelectionKeys = ref<DiagramElementUniqueKey[]>([]);
const currentSelectionKeys = computed({
  get() {
    return _rawSelectionKeys.value;
  },
  set(newSelection) {
    const sortedDeduped = _.sortBy(_.uniq(newSelection));
    // don't set the array if it's the same, helps us only care about actual changes
    if (_.isEqual(currentSelectionKeys.value, sortedDeduped)) return;
    _rawSelectionKeys.value = sortedDeduped;
  },
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
  if (!toSelect) currentSelectionKeys.value = [];
  else currentSelectionKeys.value = _.isArray(toSelect) ? toSelect : [toSelect];
}

// toggles selected items in the selection (used when shift clicking)
function toggleSelectedByKey(
  toToggle: DiagramElementUniqueKey | DiagramElementUniqueKey[],
) {
  const newval = _.xor(
    currentSelectionKeys.value,
    _.isArray(toToggle) ? toToggle : [toToggle],
  );
  currentSelectionKeys.value = newval;
}
function clearSelection() {
  currentSelectionKeys.value = [];
}
watch(currentSelectionKeys, () => {
  emit("update:selection", { elements: currentSelectionElements.value });
});
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
  _.each(nodesLocationInfo, (nodeLocation, nodeKey) => {
    const inSelectionBox = checkRectanglesOverlap(
      dragSelectStartPos.value!,
      dragSelectEndPos.value!,
      nodeLocation.topLeft,
      {
        x: nodeLocation.topLeft.x + nodeLocation.width,
        y: nodeLocation.topLeft.y + nodeLocation.height,
      },
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
}
function endDragSelect(doSelection = true) {
  dragSelectActive.value = false;
  if (doSelection) setSelectionByKey(dragSelectPreviewKeys.value);
}

// MOVING DIAGRAM ELEMENTS (nodes/groups/annotations/etc) ///////////////////////////////////////
const movedElementPositions = reactive<
  Record<DiagramElementUniqueKey, Vector2d>
>({});
const dragElementsActive = ref(false);
const currentSelectionMovableElements = computed(
  () =>
    _.filter(
      currentSelectionElements.value,
      (el) => el && "position" in el.def,
    ) as unknown as (DiagramNodeData | DiagramGroupData)[],
);
const movedElementParent = reactive<Record<DiagramElementUniqueKey, string>>(
  {},
);

const draggedElementsPositionsPreDrag = ref<
  Record<DiagramElementUniqueKey, Vector2d | undefined>
>({});
const totalScrolledDuringDrag = ref<Vector2d>({ x: 0, y: 0 });

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
  // fire off final move event, might want to clean up how this is done...
  _.each(currentSelectionMovableElements.value, (el) => {
    if (!movedElementPositions[el.uniqueKey]) return;

    const parentId =
      movedElementParent[el.uniqueKey] || el.def.parentNodeId || undefined;
    if (parentId === undefined) {
      // handle dragging items into a group
      const elShape = kStage.findOne(`#${el.uniqueKey}--bg`);
      const elPos = elShape.getAbsolutePosition(kStage);

      const elRect = {
        x: elPos.x,
        y: elPos.y,
        width: elShape.width(),
        height: elShape.height(),
      };

      const groupOrderedByZIndex = _.sortBy(groups.value, (g) => {
        const groupShape = kStage.findOne(`#${g.uniqueKey}--bg`);
        return -(groupShape?.getAbsoluteZIndex() ?? -Infinity);
      });

      const newContainingGroup = groupOrderedByZIndex.find((group) => {
        if (group.uniqueKey === el.uniqueKey) return false;

        const groupShape = kStage.findOne(`#${group.uniqueKey}--bg`);
        const groupPos = groupShape.getAbsolutePosition(kStage);

        const groupRect = {
          x: groupPos.x,
          y: groupPos.y,
          width: groupShape.width(),
          height: groupShape.height(),
        };

        return rectContainsAnother(elRect, groupRect);
      });
      if (
        newContainingGroup &&
        el.def.parentNodeId !== newContainingGroup.def.id
      ) {
        emit("group-elements", {
          group: newContainingGroup,
          elements: [el],
        });
        movedElementParent[el.uniqueKey] = newContainingGroup.def.id;
      }
    }

    const movedElementPosition = movedElementPositions[el.uniqueKey];
    if (movedElementPosition) {
      // move the element itself
      emit("move-element", {
        element: el,
        position: movedElementPosition,
        isFinal: true,
      });
    }

    // move child elements inside of a group
    if (el instanceof DiagramGroupData) {
      // for now only dealing with nodes... will be fixed later
      const childEls = _.filter(
        nodes.value,
        (n) => n.def.parentNodeId === el.def.id,
      );
      _.each(childEls, (childEl) => {
        emit("move-element", {
          element: childEl,
          // Again, not sure how we should handle a possible missing element
          // position
          position: movedElementPositions[childEl.uniqueKey] ?? { x: 0, y: 0 },
          isFinal: true,
        });
      });
    }
  });
}

let dragToEdgeScrollInterval: ReturnType<typeof setInterval> | undefined;
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

  _.each(currentSelectionMovableElements.value, (el) => {
    if (!draggedElementsPositionsPreDrag.value?.[el.uniqueKey]) return;
    const newPosition = vectorAdd(
      draggedElementsPositionsPreDrag.value[el.uniqueKey]!,
      delta,
    );

    // block moving components outside of their group
    if (el.def.parentNodeId) {
      const parentGroup = getElementByKey(
        DiagramGroupData.generateUniqueKey(el.def.parentNodeId),
      );
      if (!parentGroup) throw new Error("parent group not found");

      const groupShape = kStage.findOne(`#${parentGroup?.uniqueKey}--bg`);
      const groupPos = groupShape.getAbsolutePosition(kStage);
      const groupBounds = {
        left: groupPos.x + GROUP_INTERNAL_PADDING,
        right: groupPos.x + groupShape.width() - GROUP_INTERNAL_PADDING,
        top:
          groupPos.y +
          GROUP_INTERNAL_PADDING +
          (el.def.nodeType === "component"
            ? 0
            : GROUP_HEADER_ICON_SIZE +
              GROUP_HEADER_BOTTOM_MARGIN +
              GROUP_INTERNAL_PADDING / 2),
        bottom:
          groupPos.y + groupShape.height() - GROUP_BOTTOM_INTERNAL_PADDING,
      };

      const elShape = kStage.findOne(`#${el.uniqueKey}--bg`);
      // const elPos = elShape.getAbsolutePosition(kStage);
      const newElBounds = {
        left: newPosition.x - elShape.width() / 2,
        right: newPosition.x + elShape.width() / 2,
        top: newPosition.y,
        bottom: newPosition.y + elShape.height(),
      };

      if (newElBounds.left <= groupBounds.left) {
        newPosition.x = groupBounds.left + elShape.width() / 2;
      }
      if (newElBounds.right >= groupBounds.right) {
        newPosition.x = groupBounds.right - elShape.width() / 2;
      }
      if (newElBounds.top <= groupBounds.top) {
        newPosition.y = groupBounds.top;
      }
      if (newElBounds.bottom >= groupBounds.bottom) {
        newPosition.y = groupBounds.bottom - elShape.height();
      }
    }

    if (el instanceof DiagramGroupData) {
      const includedGroups: DiagramNodeData[] & DiagramGroupData[] = [];
      const queue = [el];
      while (queue.length > 0) {
        const parent = queue.shift();
        const x = _.filter(
          groups.value,
          (n) => n.def.parentNodeId === parent?.def.id,
        );
        _.each(x, (childGroup) => {
          queue.push(childGroup);
          includedGroups.push(childGroup);
        });
      }

      const nodeChildrenOfGroups = _.filter(
        nodes.value,
        (n) =>
          _.find(includedGroups, (g) => g.def.id === n.def.parentNodeId) !==
          undefined,
      );

      const childEls = _.concat(
        _.filter(nodes.value, (n) => n.def.parentNodeId === el.def.id),
        includedGroups,
        nodeChildrenOfGroups,
      );

      const actualParentDelta = vectorBetween(
        draggedElementsPositionsPreDrag.value[el.uniqueKey]!,
        newPosition,
      );

      // TODO: this should get simplified once we are storing positions relative to their group parent
      _.each(childEls, (childEl) => {
        if (!draggedElementsPositionsPreDrag.value?.[childEl.uniqueKey]) return;

        const newChildPosition = vectorAdd(
          draggedElementsPositionsPreDrag.value[childEl.uniqueKey]!,
          actualParentDelta,
        );
        // track the position locally, so we don't need to rely on parent to store the temporary position
        movedElementPositions[childEl.uniqueKey] = newChildPosition;
        emit("move-element", {
          element: childEl,
          position: newChildPosition,
          isFinal: false,
        });
      });
    }

    // track the position locally, so we don't need to rely on parent to store the temporary position
    movedElementPositions[el.uniqueKey] = newPosition;
    emit("move-element", {
      element: el,
      position: newPosition,
      isFinal: false,
    });
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
  _.each(currentSelectionMovableElements.value, (el) => {
    const newPosition = {
      x: alignedX === undefined ? el.def.position.x : alignedX,
      y: alignedY === undefined ? el.def.position.y : alignedY,
    };
    movedElementPositions[el.uniqueKey] = newPosition;
    emit("move-element", {
      element: el,
      position: newPosition,
      isFinal: true,
    });
  });
  // TODO: move viewport to show selection
}
function nudgeSelection(direction: Direction, largeNudge: boolean) {
  if (!currentSelectionMovableElements.value.length) return;
  const nudgeSize = largeNudge ? 10 : 1;
  const nudgeVector: Vector2d = {
    left: { x: -1 * nudgeSize, y: 0 },
    right: { x: 1 * nudgeSize, y: 0 },
    up: { x: 0, y: -1 * nudgeSize },
    down: { x: 0, y: 1 * nudgeSize },
  }[direction];
  _.each(currentSelectionMovableElements.value, (el) => {
    const newPosition = vectorAdd(el.def.position, nudgeVector);
    movedElementPositions[el.uniqueKey] = newPosition;
    emit("move-element", {
      element: el,
      position: newPosition,
      isFinal: true,
    });
  });
  // TODO: if nudging out of the viewport, pan to give more space
}

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
watch([resizedElementSizes, isMounted, movedElementPositions, stageRef], () => {
  if (!kStage) return;

  const boxDictionary: Record<string, IRect> = {};

  for (const group of groups.value) {
    const childIds = group.def.childNodeIds;
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

      const geometry = {
        x: position.x,
        y: position.y,
        width: elShape.width(),
        height: elShape.height(),
      };

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
});

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
  if (!resizeElement.value) return;
  const size = resizedElementSizes[resizeElement.value.uniqueKey];
  const position = movedElementPositions[resizeElement.value.uniqueKey];
  if (!size || !position) {
    return;
  }

  emit("resize-element", {
    element: resizeElement.value,
    position,
    size,
    isFinal: true,
  });

  resizeElement.value = undefined;
}

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

  const minNodeDimension = NODE_WIDTH + 20 * 2;
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
        const minDelta = minNodeDimension - presentSize.height;
        if (sizeDelta.y < minDelta) {
          sizeDelta.y = minDelta;
        }
      }
      break;
    case "left":
      {
        sizeDelta.y = 0;
        sizeDelta.x = -sizeDelta.x;
        const minDelta = minNodeDimension - presentSize.width;
        if (sizeDelta.x < minDelta) {
          sizeDelta.x = minDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "right":
      {
        sizeDelta.y = 0;
        const minDelta = minNodeDimension - presentSize.width;
        if (sizeDelta.x < minDelta) {
          sizeDelta.x = minDelta;
        }
        positionDelta.x = sizeDelta.x;
      }
      break;
    case "bottom-left":
      {
        const minYDelta = minNodeDimension - presentSize.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }

        sizeDelta.x = -sizeDelta.x;
        const minXDelta = minNodeDimension - presentSize.width;
        if (sizeDelta.x < minXDelta) {
          sizeDelta.x = minXDelta;
        }
        positionDelta.x = -sizeDelta.x;
      }
      break;
    case "bottom-right":
      {
        const minYDelta = minNodeDimension - presentSize.height;
        if (sizeDelta.y < minYDelta) {
          sizeDelta.y = minYDelta;
        }
        const minXDelta = minNodeDimension - presentSize.width;
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
  const parentId = movedElementParent[resizeTargetKey] || node.parentNodeId;

  if (parentId) {
    // Resized element with top-left corner xy coordinates instead of top-center
    const newNodeRect = {
      ...newNodePosition,
      ...newNodeSize,
      x: newNodePosition.x - newNodeSize.width / 2,
    };

    const parent = groups.value.find((g) => g.def.id === parentId);
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

  resizedElementSizes[resizeTargetKey] = newNodeSize;
  movedElementPositions[resizeTargetKey] = newNodePosition;
  emit("resize-element", {
    element: resizeElement.value,
    position: newNodePosition,
    size: newNodeSize,
    isFinal: false,
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

    // now check socket "types"
    // TODO: probably will rework this - maybe use same type, or use edge types?
    return fromSocket.def.type === possibleToSocket.def.type;
  });
  return _.map(possibleSockets, (s) => s.uniqueKey);
});

const drawEdgeState = computed(() => {
  return {
    active: drawEdgeActive.value,
    fromSocketKey: drawEdgeFromSocketKey.value,
    toSocketKey: drawEdgeToSocketKey.value,
    possibleTargetSocketKeys: drawEdgePossibleTargetSocketKeys.value,
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

  emit("draw-edge", {
    fromSocket: adjustedFrom,
    toSocket: adjustedTo,
  });
}
// ELEMENT ADDITION
const insertElementActive = ref(false);
const insertElementType = ref<DiagramElementTypes>();
const pendingInsertedElements = reactive<
  Record<string, PendingInsertedElement>
>({});
function beginInsertElement(elementType: DiagramElementTypes) {
  clearSelection();
  insertElementActive.value = true;
  insertElementType.value = elementType;
  // TODO: this will likely need more info as subtypes emerge
  // ie inserting an X-node vs Y-node, or annotation of a specific type
}
function endInsertElement() {
  insertElementActive.value = false;
}
function triggerInsertElement() {
  if (!insertElementActive.value || !insertElementType.value)
    throw new Error("insert element mode must be active");
  if (!gridPointerPos.value)
    throw new Error("Cursor must be in grid to insert element");

  let parentGroupId;
  if (hoveredElement.value instanceof DiagramGroupData) {
    parentGroupId = hoveredElement.value.def.id;
  }

  const insertId = _.uniqueId("insert-diagram-el");
  pendingInsertedElements[insertId] = {
    diagramElementType: insertElementType.value,
    insertedAt: new Date(),
    position: gridPointerPos.value,
  };
  // we need a way to know when the insert is complete
  // ideally without trying to match up new data (nodes/etc) that comes in through props
  // because in multiplayer mode we may have new stuff flowing in
  // so we pass a callback for the parent to call when the insert is done
  emit("insert-element", {
    diagramElementType: insertElementType.value,
    position: gridPointerPos.value,
    parent: parentGroupId,
    onComplete: () => {
      delete pendingInsertedElements[insertId];
    },
  });
  endInsertElement();
}

// ELEMENT DELETION ////////////////////////////////////////////////////////////////////
function deleteSelected() {
  if (!currentSelectionElements.value?.length) return;
  const selected = currentSelectionElements.value;
  if (!selected) {
    return;
  }

  // previously we were deleting edges connected to nodes from here
  // but we may want to handle this purely from the backend?
  // // when deleting a node, we also have to delete any attached edges
  // const additionalEdgesToDelete = _.flatMap(selected, (el) => {
  //   if (el instanceof DiagramNodeData) return [];
  //   return _.flatMap(connectedEdgesByElementKey.value[el.uniqueKey]);
  // });
  // // have to dedupe in case we are deleting both nodes connected to an edge
  // const uniqueEdgesToDelete = _.uniq(additionalEdgesToDelete);
  emit("delete-elements", { elements: selected as DiagramElementData[] });
}

// LAYOUT REGISTRY + HELPERS ///////////////////////////////////////////////////////////
type NodeLocationInfo = { topLeft: Vector2d; width: number; height: number };
type SocketLocationInfo = { center: Vector2d };
const nodesLocationInfo = reactive<Record<string, NodeLocationInfo>>({});
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
    topLeft: nodeBgShape.getAbsolutePosition(kStage),
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

// const nodes = ref([] as DiagramNode[]);
// const sockets = ref([] as DiagramSocket[]);
// const edges = ref([] as DiagramEdge[]);

const nodes = computed(() =>
  _.map(
    _.filter(props.nodes, (n) => n.nodeType === "component"),
    (nodeDef) => new DiagramNodeData(nodeDef),
  ),
);
const groups = computed(() =>
  _.map(
    _.filter(props.nodes, (n) => n.nodeType !== "component"),
    (groupDef) => new DiagramGroupData(groupDef),
  ),
);
const sockets = computed(() =>
  _.compact(_.flatMap(_.concat(nodes.value, groups.value), (i) => i.sockets)),
);
const edges = computed(() =>
  _.map(props.edges, (edgeDef) => new DiagramEdgeData(edgeDef)),
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

function recenter() {
  gridOrigin.value = { x: 0, y: 0 };
  zoomLevel.value = 1;
}

const diagramConfig = computed(() => {
  return _.merge(baseConfig, props.customConfig);
});

// set up provider so children can grab config without needing to pass down through many levels
useDiagramConfigProvider(diagramConfig);
useZoomLevelProvider(zoomLevel);

// functions exposed to outside world ///////////////////////////////////
defineExpose({
  setZoom,
  recenter,
  setSelectionByKey,
  setHoveredByKey,
  clearSelection,
  beginInsertElement,
  endInsertElement,
});

const helpModalRef = ref();
</script>
