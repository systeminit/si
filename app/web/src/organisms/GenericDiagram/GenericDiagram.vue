Generic diagram component * NOTE - uses a resize observer to react to size
changes, so this must be placed in a container that is sized explicitly has
overflow hidden */
<template>
  <div ref="containerRef" class="absolute inset-0 overflow-hidden">
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
    <DiagramControls :zoom-level="zoomLevel" @update:zoom="setZoom" />
    <v-stage
      v-if="customFontsLoaded"
      ref="stageRef"
      :config="{
        width: containerWidth,
        height: containerHeight,
        scale: { x: zoomLevel, y: zoomLevel },
        offset: { x: gridMinX, y: gridMinY },
      }"
      class=""
      :style="{ cursor }"
      @mousedown="onMouseDown"
    >
      <DiagramGridBackground
        :grid-min-x="gridMinX"
        :grid-max-x="gridMaxX"
        :grid-min-y="gridMinY"
        :grid-max-y="gridMaxY"
        :zoom-level="zoomLevel"
      />
      <v-layer>
        <DiagramNode
          v-for="node in nodes"
          :key="node.id"
          :node="node"
          :temp-position="movedElementPositions[node.id]"
          :connected-edges="connectedEdgesByNodeIdBySocketId[node.id]"
          :draw-edge-state="drawEdgeState"
          :is-hovered="elementIsHovered('node', node.id)"
          :is-selected="elementIsSelected('node', node.id)"
          @hover:start="(socketId) => onNodeHoverStart(node.id, socketId)"
          @hover:end="(socketId) => onNodeHoverEnd(node.id, socketId)"
          @resize="onNodeLayoutOrLocationChange(node.id)"
        />
        <DiagramEdge
          v-for="edge in edges"
          :key="edge.id"
          :edge="edge"
          :from-point="socketsLocationInfo[edge.fromSocketId]?.center"
          :to-point="socketsLocationInfo[edge.toSocketId]?.center"
          :is-hovered="elementIsHovered('edge', edge.id)"
          :is-selected="elementIsSelected('edge', edge.id)"
          @hover:start="onEdgeHoverStart(edge.id)"
          @hover:end="onEdgeHoverEnd(edge.id)"
        />
        <!-- placeholders for new inserted elements still processing -->
        <v-rect
          v-for="(pendingInsert, pendingInsertId) in pendingInsertedElements"
          :key="pendingInsertId"
          :config="{
            width: 100,
            height: 50,
            cornerRadius: CORNER_RADIUS,
            x: pendingInsert.position!.x - 50,
            y: pendingInsert.position!.y,
            fill: 'rgba(0,0,0,.4)'
          }"
        />
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
          :from-point="socketsLocationInfo[drawEdgeFromSocketId!]?.center"
          :to-point="drawEdgeToSocketId ? socketsLocationInfo[drawEdgeToSocketId!]?.center : gridPointerPos"
        />
      </v-layer>
    </v-stage>
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
import _ from "lodash";
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d } from "konva/lib/types";
import tinycolor from "tinycolor2";
import { useCustomFontsLoaded } from "@/composables/useFontLoaded";
import DiagramGridBackground from "./DiagramGridBackground.vue";
import {
  DeleteElementsEvent,
  DiagramConfig,
  DiagramDrawEdgeState,
  DiagramEdgeDef,
  DiagramElementIdentifier,
  DiagramNodeDef,
  DrawEdgeEvent,
  MoveElementEvent,
  Direction,
  PendingInsertedElement,
  DiagramElementTypes,
  InsertElementEvent,
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
} from "./diagram_constants";
import {
  vectorDistance,
  vectorAdd,
  checkRectanglesOverlap,
} from "./utils/math";
import DiagramNewEdge from "./DiagramNewEdge.vue";
import { convertArrowKeyToDirection } from "./utils/keyboard";
import DiagramControls from "./DiagramControls.vue";

import { baseConfig } from "./diagram_base_config";

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
});

const emit = defineEmits<{
  (e: "update:zoom", newZoom: number): void;
  (e: "update:selection", newSelection: DiagramElementIdentifier[]): void;
  (e: "move-element", nodeMoveInfo: MoveElementEvent): void;
  (e: "delete-elements", deleteInfo: DeleteElementsEvent): void;
  (e: "insert-element", insertInfo: InsertElementEvent): void;
  (e: "draw-edge", drawEdgeInfo: DrawEdgeEvent): void;
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
  if (newZoomLevel < MIN_ZOOM || newZoomLevel > MAX_ZOOM) return;
  zoomLevel.value = newZoomLevel;
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
  resizeObserver.observe(containerRef.value!);
}

onBeforeUnmount(() => {
  kStage.off("wheel", onMouseWheel);
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
  // TODO: check is cursor is within graph bounds
  // TODO: check if something else (like an input) is focused and bail

  // if focused on an input (or anything) dont do anything, let normal behaviour proceed
  // TODO: this should be more sophisticated
  if (document?.activeElement?.tagName !== "BODY") return;

  // console.log(e);

  // handle arrow keys - nudge and alignment
  if (e.key.startsWith("Arrow")) {
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
  if (e.key === "Delete" || e.key === "Backspace") deleteSelected();
}
function onKeyUp(e: KeyboardEvent) {
  if (e.key === " ") spaceKeyIsDown.value = false;
  if (e.key === "Shift") shiftKeyIsDown.value = false;
}

const mouseIsDown = ref(false);
const dragThresholdBroken = ref(false);
const lastMouseDownEvent = ref<MouseEvent>();
const lastMouseDownContainerPointerPos = ref<Vector2d>();
const lastMouseDownDiagramElement = ref<DiagramElementIdentifier>();
function onMouseDown(ke: KonvaEventObject<MouseEvent>) {
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
  lastMouseDownDiagramElement.value = hoveredElement.value;

  // for drag to pan, we start dragging right away since the user has enabled it by holding the space bar
  // for all other interactions, we watch to see if the user drags past some small threshold to begin the "drag"
  // in order to ignore clicks with a tiny bit of movement
  if (dragToPanArmed.value) beginDragToPan();
  else if (insertElementActive.value) triggerInsertElement();
  else handleMouseDownSelection();
}
function onMouseUp(e: MouseEvent) {
  // we only care here about left click - might change this later...
  if (e.button !== 0) return;
  mouseIsDown.value = false;
  if (dragToPanActive.value) endDragToPan();
  else if (dragElementsActive.value) endDragElements();
  else if (dragSelectActive.value) endDragSelect();
  else if (drawEdgeActive.value) endDrawEdge();
  // we'll handle insert on mouseup too in case the user dragged the element from the asset palette and then let go in the canvas
  // TODO: probably change this - its a bit hacky...
  else if (insertElementActive.value && pointerIsWithinGrid.value)
    triggerInsertElement();
  else handleMouseUpSelection();
}
function onMouseMove(e: MouseEvent) {
  // update pointer location relative to container, which is used throughout
  containerPointerPos.value = {
    x: e.clientX - containerViewportX.value,
    y: e.clientY - containerViewportY.value,
  };

  if (dragToPanActive.value) onDragToPanMove();
  else if (dragElementsActive.value) onDragElementsMove();
  else if (dragSelectActive.value) onDragSelectMove();
  else if (drawEdgeActive.value) onDrawEdgeMove();
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
  if (!lastMouseDownDiagramElement.value) {
    // begin drag to multi-select
    beginDragSelect();
  } else if (props.readOnly) {
    // TODO: add controls for each of these modes...
    return;
  } else if (
    lastMouseDownDiagramElement.value.diagramElementType === "socket"
  ) {
    // begin drawing edge
    beginDrawEdge(lastMouseDownDiagramElement.value.id);
  } else if (lastMouseDownDiagramElement.value.diagramElementType === "edge") {
    // not sure what dragging an edge means... maybe nothing?
    console.log("dragging edge ?");
  } else if (lastMouseDownDiagramElement.value.diagramElementType === "node") {
    // begin moving selected nodes (and eventually other movable things like groups / annotations, etc...)
    beginDragElements();
  }
}

// Mode and cursor
const cursor = computed(() => {
  if (dragToPanActive.value) return "grabbing";
  if (dragToPanArmed.value) return "grab";
  if (dragSelectActive.value) return "crosshair";

  if (
    !props.readOnly &&
    hoveredElement.value?.diagramElementType === "socket"
  ) {
    return "cell";
  }
  if (drawEdgeActive.value) return "cell";
  if (dragElementsActive.value) return "move";
  if (insertElementActive.value) return "copy"; // not sure about this...
  return "auto";
});

// node hovering

// we track these things separately rather than as a single general item because hovering a socket also means hovering over the node
const hoveredElement = ref<DiagramElementIdentifier>();
// same event and handler is used for both hovering nodes and sockets
// NOTE - we'll receive 2 events when hovering sockets, one for the node and one for the socket
function onNodeHoverStart(nodeId: string, socketId?: string) {
  if (socketId) {
    hoveredElement.value = { diagramElementType: "socket", id: socketId };
  } else {
    hoveredElement.value = { diagramElementType: "node", id: nodeId };
  }
}
function onNodeHoverEnd(_nodeId: string, _socketId?: string) {
  hoveredElement.value = undefined;
}
function onEdgeHoverStart(edgeId: string) {
  hoveredElement.value = { diagramElementType: "edge", id: edgeId };
}
function onEdgeHoverEnd(_edgeId: string) {
  hoveredElement.value = undefined;
}
const disableHoverEvents = computed(() => {
  if (dragToPanArmed.value || dragToPanActive.value) return true;
  if (dragElementsActive.value) return true;
  if (dragSelectActive.value) return true;
  if (drawEdgeActive.value) return true;
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

// ELEMENT SELECTION /////////////////////////////////////////////////////////////////////////////////
const _rawSelection = ref<DiagramElementIdentifier[]>([]);
const currentSelection = computed({
  get() {
    return _rawSelection.value;
  },
  set(newSelection) {
    const sortedDeduped = _.sortBy(_.uniq(newSelection), "id");
    // dont set the array if its the same, helps us only care about actual changes
    if (_.isEqual(currentSelection.value, sortedDeduped)) return;
    _rawSelection.value = sortedDeduped;
  },
});
function setSelection(
  toSelect: DiagramElementIdentifier | DiagramElementIdentifier[],
) {
  currentSelection.value = _.isArray(toSelect) ? toSelect : [toSelect];
}
// toggles selected items in the selection (used when shift clicking)
function toggleSelected(
  toToggle: DiagramElementIdentifier | DiagramElementIdentifier[],
) {
  const newval = _.xorBy(
    currentSelection.value,
    _.isArray(toToggle) ? toToggle : [toToggle],
    // comparator helper - this is a bit weird but it's not a fn to compare 2 items, its a fn to create a value used in the comparison
    (el) => `${el.diagramElementType}--${el.id}`,
  );
  currentSelection.value = newval;
}
function clearSelection() {
  currentSelection.value = [];
}
watch(currentSelection, () => {
  emit("update:selection", currentSelection.value);
});
function elementIsHovered(diagramElementType: string, id: string) {
  return (
    !disableHoverEvents.value &&
    _.isEqual(hoveredElement.value, { diagramElementType, id })
  );
}
function elementIsSelected(diagramElementType: string, id: string) {
  if (dragSelectActive.value) {
    return !!_.find(dragSelectSelectionPreview.value, {
      diagramElementType,
      id,
    });
  } else {
    return !!_.find(currentSelection.value, { diagramElementType, id });
  }
}

const handleSelectionOnMouseUp = ref(false);
function handleMouseDownSelection() {
  handleSelectionOnMouseUp.value = false;
  // handle clicking nothing / background grid
  if (!hoveredElement.value) {
    // we clear selection on mousedown unless shift is held
    // in which case it could be beginning of drag to select, so we handle on mouseup
    if (!shiftKeyIsDown.value) clearSelection();
    else handleSelectionOnMouseUp.value = true;
    return;
  }

  // nodes can be multi-selected, so we have some extra behaviour
  // TODO: other elements may also share this behavoiur
  if (hoveredElement.value.diagramElementType === "node") {
    // when clicking on an element that is NOT currently selected, we act right away
    // but if the element IS selected, this could be beginning of dragging
    // so we handle selection on mouseup if the user never fully started to drag
    if (!_.find(currentSelection.value, hoveredElement.value)) {
      if (shiftKeyIsDown.value) toggleSelected(hoveredElement.value);
      else setSelection(hoveredElement.value);
    } else {
      handleSelectionOnMouseUp.value = true;
    }
  } else {
    setSelection(hoveredElement.value);
  }
}

// handles selection on mouseup for scenarios where the user _might_ have started dragging but did not
// see handleMouseDownSelection() for when those take place
// NOTE - this only fires if the user never breaks the drag distance threshold
function handleMouseUpSelection() {
  if (!handleSelectionOnMouseUp.value || dragThresholdBroken.value) return;
  const clickedEl = lastMouseDownDiagramElement.value;

  if (!clickedEl) clearSelection();
  else if (lastMouseDownEvent.value?.shiftKey) toggleSelected(clickedEl);
  else setSelection(clickedEl);
}

// DRAG SELECT BOX //////////////////////////////////////////////////////
const dragSelectActive = ref(false);
const dragSelectStartPos = ref<Vector2d>();
const dragSelectEndPos = ref<Vector2d>();
const dragSelectSelectionPreview = ref<DiagramElementIdentifier[]>([]);
const SELECTION_BOX_INNER_COLOR = tinycolor(SELECTION_COLOR)
  .setAlpha(0.4)
  .toRgbString();
function beginDragSelect() {
  if (!containerPointerPos.value) return;
  dragSelectSelectionPreview.value = [];
  dragSelectActive.value = true;
  // this triggers after the user breaks the dragging threshold, so we dont start at curent position, but where they clicked
  dragSelectStartPos.value = convertContainerCoordsToGridCoords(
    containerPointerPos.value,
  );
  dragSelectEndPos.value = undefined;
}
function onDragSelectMove() {
  dragSelectEndPos.value = gridPointerPos.value;

  const selectedInBox: DiagramElementIdentifier[] = [];
  _.each(nodesLocationInfo, (nodeLocation, nodeId) => {
    const inSelectionBox = checkRectanglesOverlap(
      dragSelectStartPos.value!,
      dragSelectEndPos.value!,
      nodeLocation.topLeft,
      {
        x: nodeLocation.topLeft.x + nodeLocation.width,
        y: nodeLocation.topLeft.y + nodeLocation.height,
      },
    );
    if (inSelectionBox)
      selectedInBox.push({ diagramElementType: "node", id: nodeId });
  });
  // if holding shift key, we'll add/toggle the existing selection with whats in the box
  // NOTE - weird edge cases around what if you let go of shift after beginning the drag which we are ignoring
  if (lastMouseDownEvent.value?.shiftKey) {
    dragSelectSelectionPreview.value = _.xorBy(
      currentSelection.value,
      selectedInBox,
      // see note in toggleSelected()
      (el) => `${el.diagramElementType}--${el.id}`,
    );
  } else {
    dragSelectSelectionPreview.value = selectedInBox;
  }
}
function endDragSelect(doSelection = true) {
  dragSelectActive.value = false;
  if (doSelection) setSelection(dragSelectSelectionPreview.value);
}

// MOVING DIAGRAM ELEMENTS (nodes/groups/annotations/etc) ///////////////////////////////////////
const movedElementPositions = reactive<Record<string, Vector2d>>({});
const dragElementsActive = ref(false);
const currentSelectionMovableElements = computed(() => {
  const draggableElIds = _.filter(
    currentSelection.value,
    (el) => el.diagramElementType === "node",
  );
  const draggableEls = _.compact(_.map(draggableElIds, getElement));
  return draggableEls as DiagramNodeDef[];
});

const draggedElementsPositionsPreDrag = ref<Vector2d[]>();
const totalScrolledDuringDrag = ref<Vector2d>({ x: 0, y: 0 });
function beginDragElements() {
  if (!lastMouseDownDiagramElement.value) return;
  dragElementsActive.value = true;
  // TODO: better type here... wanting to use the helper which can return anything, but it will only be a "movable" element
  // const draggedElement = getElement(
  //   lastMouseDownDiagramElement.value,
  // ) as DiagramNodeDef;

  totalScrolledDuringDrag.value = { x: 0, y: 0 };

  draggedElementsPositionsPreDrag.value = _.map(
    currentSelectionMovableElements.value,
    (el) => movedElementPositions[el.id] || el.position!,
  );
}
function endDragElements() {
  dragElementsActive.value = false;
  // fire off final move event, might want to clean up how this is done...
  _.each(currentSelectionMovableElements.value, (el) => {
    if (!movedElementPositions[el.id]) return;
    emit("move-element", {
      id: el.id,
      diagramElementType: "node",
      position: movedElementPositions[el.id],
      isFinal: true,
    });

    // TODO: probably should remove the temp position, but since the backend is super slow, we'll leave it for now
    // delete movedElementPositions[el.id];
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

  _.each(currentSelectionMovableElements.value, (el, i) => {
    if (!draggedElementsPositionsPreDrag.value?.[i]) return;
    const newPosition = vectorAdd(
      draggedElementsPositionsPreDrag.value?.[i],
      delta,
    );
    // track the position locally, so we dont need to rely on parent to store the temporary position
    movedElementPositions[el.id] = newPosition;
    emit("move-element", {
      id: el.id,
      diagramElementType: "node",
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

  // track total amount scrolled becuase we need to offset from original drag click location
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
    (el) => el.position,
  );
  const xPositions = _.map(positions, (p) => p.x);
  const yPositions = _.map(positions, (p) => p.y);
  if (direction === "up") alignedY = _.min(yPositions);
  else if (direction === "down") alignedY = _.max(yPositions);
  else if (direction === "left") alignedX = _.min(xPositions);
  else if (direction === "right") alignedX = _.max(xPositions);
  _.each(currentSelectionMovableElements.value, (el) => {
    const newPosition = {
      x: alignedX === undefined ? el.position.x : alignedX,
      y: alignedY === undefined ? el.position.y : alignedY,
    };
    movedElementPositions[el.id] = newPosition;
    emit("move-element", {
      id: el.id,
      diagramElementType: "node",
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
    const newPosition = vectorAdd(el.position, nudgeVector);
    movedElementPositions[el.id] = newPosition;
    emit("move-element", {
      id: el.id,
      diagramElementType: "node",
      position: newPosition,
      isFinal: true,
    });
  });
  // TODO: if nudging out of the viewport, pan to give more space
}

// DRAWING EDGES ///////////////////////////////////////////////////////////////////////
const drawEdgeActive = ref(false);
const drawEdgeFromSocketId = ref<string>();
const drawEdgeToSocketId = ref<string>();
const drawEdgeTargetSocketIds = computed(() => {
  if (!drawEdgeActive.value || !drawEdgeFromSocketId.value) return [];
  const fromSocket = allSocketsById.value[drawEdgeFromSocketId.value];
  const fromSocketNodeId = socketIdToNodeIdLookup.value[fromSocket.id].id;
  const fromSocketExistingEdges =
    connectedEdgesByNodeIdBySocketId.value[fromSocketNodeId]?.[fromSocket.id];
  const alreadyConnectedSocketIds = _.map(fromSocketExistingEdges, (edge) => {
    return edge.fromSocketId === fromSocket.id
      ? edge.toSocketId
      : edge.fromSocketId;
  });
  // check each socket if it's a possible "to" target to create an edge
  const possibleToSockets = _.filter(allSockets.value, (toSocket) => {
    // sockets cannot connect to other sockets on the same node (for now - may want to revisit this)
    const toSocketNodeId = socketIdToNodeIdLookup.value[toSocket.id].id;
    if (fromSocketNodeId === toSocketNodeId) return false;
    // cannot connect to a socket that is already connected
    if (alreadyConnectedSocketIds.includes(toSocket.id)) return false;
    if (fromSocket.direction === "input" && toSocket.direction === "input")
      return false;
    if (fromSocket.direction === "output" && toSocket.direction === "output")
      return false;

    // now check socket "types"
    // TODO: probably will rework this - maybe use same type, or use edge types?
    return fromSocket.type === toSocket.type;
  });
  return _.map(possibleToSockets, (s) => s.id);
});
const drawEdgeState = computed(() => {
  const state: DiagramDrawEdgeState = {
    active: drawEdgeActive.value,
    fromSocketId: drawEdgeFromSocketId.value,
    toSocketId: drawEdgeToSocketId.value,
    targetSocketIds: drawEdgeTargetSocketIds.value,
  };
  return state;
});

function beginDrawEdge(fromSocketId: string) {
  drawEdgeActive.value = true;
  drawEdgeFromSocketId.value = fromSocketId;
}
function onDrawEdgeMove() {
  if (!gridPointerPos.value) return;
  // look through the possible target sockets, and find distances to the pointer
  const socketPointerDistances = _.map(
    drawEdgeTargetSocketIds.value,
    (socketId: string) => ({
      id: socketId,
      pointerDistance: vectorDistance(
        gridPointerPos.value!,
        socketsLocationInfo[socketId]?.center,
      ),
    }),
  );
  const nearest = _.minBy(socketPointerDistances, (d) => d.pointerDistance);
  // give a little buffer so the pointer will magnet to nearby sockets
  if (nearest && nearest.pointerDistance < SOCKET_SIZE * 2) {
    drawEdgeToSocketId.value = nearest.id;
  } else {
    drawEdgeToSocketId.value = undefined;
  }
}
async function endDrawEdge() {
  drawEdgeActive.value = false;
  if (!drawEdgeFromSocketId.value || !drawEdgeToSocketId.value) return;

  // if the user dragged from an input to an output, we'll reverse the direction when we fire off the event
  const swapDirection =
    allSocketsById.value[drawEdgeFromSocketId.value].direction === "input";
  emit("draw-edge", {
    fromSocketId: swapDirection
      ? drawEdgeToSocketId.value
      : drawEdgeFromSocketId.value,
    toSocketId: swapDirection
      ? drawEdgeFromSocketId.value
      : drawEdgeToSocketId.value,
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
    onComplete: () => {
      delete pendingInsertedElements[insertId];
    },
  });
  endInsertElement();
}

// ELEMENT DELETION ////////////////////////////////////////////////////////////////////
function deleteSelected() {
  if (!currentSelection.value?.length) return;
  const selected = currentSelection.value;
  // when deleting a node, we also have to delete any attached edges
  const additionalEdgesToDelete = _.flatMap(selected, (el) => {
    if (el.diagramElementType !== "node") return [];
    return _.flatMap(connectedEdgesByNodeIdBySocketId.value[el.id]);
  });
  // have to dedupe in case we are deleting both nodes connected to an edge
  const uniqueEdgesToDelete = _.uniqBy(additionalEdgesToDelete, (e) => e.id);
  const edgeEls: DiagramElementIdentifier[] = _.map(
    uniqueEdgesToDelete,
    (e) => ({
      diagramElementType: "edge",
      id: e.id,
    }),
  );
  emit("delete-elements", { elements: [...selected, ...edgeEls] });
}

// LAYOUT REGISTRY + HELPERS ///////////////////////////////////////////////////////////
type NodeLocationInfo = { topLeft: Vector2d; width: number; height: number };
type SocketLocationInfo = { center: Vector2d };
const nodesLocationInfo = reactive<Record<string, NodeLocationInfo>>({});
const socketsLocationInfo = reactive<Record<string, SocketLocationInfo>>({});

function onNodeLayoutOrLocationChange(nodeId: string) {
  // record node location/dimensions (used when drawing selection box)
  // we find the background shape, because the parent group has no dimensions
  const nodeBgShape = kStage.find(`#node-${nodeId}--bg`)?.[0];
  nodesLocationInfo[nodeId] = {
    topLeft: nodeBgShape.getAbsolutePosition(kStage),
    width: nodeBgShape.width(),
    height: nodeBgShape.height(),
  };

  // record new socket locations (used to render edges)
  _.each(nodesById.value[nodeId].sockets, (socket) => {
    const socketShape = kStage.find(`#socket-${socket.id}`)?.[0];
    socketsLocationInfo[socket.id] = {
      center: socketShape.getAbsolutePosition(kStage),
    };
  });
}

// DIAGRAM CONTENTS HELPERS //////////////////////////////////////////////////
const nodesById = computed(() => _.keyBy(props.nodes, (n) => n.id));
const socketIdToNodeIdLookup = computed(() => {
  const lookup: Record<string, DiagramNodeDef> = {};
  _.each(props.nodes, (node) => {
    _.each(node.sockets, (socket) => {
      lookup[socket.id] = nodesById.value[node.id];
    });
  });
  return lookup;
});

const connectedEdgesByNodeIdBySocketId = computed(() => {
  const lookup: Record<string, Record<string, DiagramEdgeDef[]>> = {};
  _.each(props.edges, (edge) => {
    const fromNode = socketIdToNodeIdLookup.value[edge.fromSocketId];
    const toNode = socketIdToNodeIdLookup.value[edge.toSocketId];
    lookup[fromNode.id] ||= {};
    lookup[fromNode.id][edge.fromSocketId] ||= [];
    lookup[fromNode.id][edge.fromSocketId].push(edge);
    lookup[toNode.id] ||= {};
    lookup[toNode.id][edge.toSocketId] ||= [];
    lookup[toNode.id][edge.toSocketId].push(edge);
  });
  return lookup;
});

const allSockets = computed(() =>
  _.flatMap(props.nodes, (n) => n.sockets || []),
);
const allSocketsById = computed(() => _.keyBy(allSockets.value, (s) => s.id));

function getElement(el: DiagramElementIdentifier) {
  if (el.diagramElementType === "node") {
    return _.find(props.nodes, (n) => n.id === el.id);
  } else if (el.diagramElementType === "edge") {
    return _.find(props.edges, (e) => e.id === el.id);
  } else if (el.diagramElementType === "socket") {
    return _.find(allSockets.value, (s) => s.id === el.id);
  }
}

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
  setSelection,
  clearSelection,
  beginInsertElement,
});
</script>
