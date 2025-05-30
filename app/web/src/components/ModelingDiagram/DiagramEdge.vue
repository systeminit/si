<template>
  <v-group v-if="points && centerPoint && showEdge">
    <v-line
      :config="{
        visible: isHovered || isSelected,
        points,
        stroke: SELECTION_COLOR,
        strokeWidth: isSelected ? 9 : 5,
        listening: false,
      }"
    />
    <v-line
      :config="{
        id: edge.uniqueKey,
        points,
        stroke: strokeColor,
        strokeWidth: 2,
        hitStrokeWidth: 10,
        listening: !edge.def.isInferred,
        dash: [10, 10],
        dashEnabled: isDeleted,
        shadowColor: '#000',
        shadowBlur: 1,
        shadowEnabled: isHovered || isSelected,
      }"
      @mousedown="onMouseDown"
      @mouseout="onMouseOut"
      @mouseover="onMouseOver"
    />

    <v-group
      v-if="connectionCount"
      :config="{
        x: centerPoint.x - COUNT_ICON_SIZE / 2,
        y: centerPoint.y - COUNT_ICON_SIZE / 2,
        listening: false,
      }"
    >
      <v-rect
        :config="{
          x: 0,
          y: 0,
          width: COUNT_ICON_SIZE,
          height: COUNT_ICON_SIZE,
          fill: connectionCountColor,
          cornerRadius: 4,
        }"
      />
      <v-text
        :config="{
          x: 0,
          y: 0,
          width: COUNT_ICON_SIZE,
          height: COUNT_ICON_SIZE,
          align: 'center',
          verticalAlign: 'middle',
          text: connectionCountText,
          fontSize: connectionCount < 10 ? 12 : 10,
          fontStyle: 'bold',
          fontFamily: DIAGRAM_FONT_FAMILY,
          fill: connectionCountColor === '#000000' ? '#FFFFFF' : '#000000',
        }"
      />
    </v-group>
    <v-group
      v-else-if="
        !edge.def.isInferred &&
        (isAdded || isDeleted || willDeleteIfPendingEdgeCreated)
      "
      :config="{
        x: centerPoint.x,
        y: centerPoint.y,
        listening: false,
      }"
    >
      <template v-if="willDeleteIfPendingEdgeCreated">
        <DiagramIcon
          :color="getToneColorHex('destructive')"
          :size="20"
          icon="scissors"
        />
      </template>
      <template v-else-if="isAdded">
        <DiagramIcon
          :color="getToneColorHex('success')"
          :size="20"
          icon="plus-square"
          shadeBg
        />
      </template>
      <template v-else-if="isDeleted">
        <DiagramIcon
          :color="getToneColorHex('destructive')"
          :size="20"
          icon="minus-square"
          shadeBg
        />
      </template>
    </v-group>
  </v-group>
</template>

<script lang="ts" setup>
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d } from "konva/lib/types";
import { computed, PropType } from "vue";
import {
  COLOR_PALETTE,
  getToneColorHex,
  useTheme,
} from "@si/vue-lib/design-system";
import { isDevMode } from "@/utils/debug";
import { useViewsStore } from "@/store/views.store";
import {
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
  SOCKET_SIZE,
} from "./diagram_constants";
import { DiagramEdgeData } from "./diagram_types";
import { pointAlongLinePct, pointAlongLinePx } from "./utils/math";
import DiagramIcon from "./DiagramIcon.vue";
import { useDiagramContext } from "./ModelingDiagram.vue";

const props = defineProps({
  edge: {
    type: Object as PropType<DiagramEdgeData>,
    required: true,
  },
  connectionCount: {
    type: Number,
  },
  fromPoint: {
    type: Object as PropType<Vector2d>,
    default: undefined,
  },
  toPoint: {
    type: Object as PropType<Vector2d>,
  },
  isHovered: Boolean,
  isSelected: Boolean,
});

const { theme } = useTheme();

const COUNT_ICON_SIZE = 16;
const connectionCountColor = computed(() => {
  const neutral = theme.value === "dark" ? "FFFFFF" : "000000";
  if (!props.edge.def.changeStatus) {
    return neutral;
  }

  const colors = {
    added: "4ADE80",
    deleted: "EF4444",
    modified: "F59E0B",
    unmodified: neutral,
  };

  return `#${colors[props.edge.def.changeStatus]}`;
});
const connectionCountText = computed(() => {
  if (!props.connectionCount) return "";
  else if (props.connectionCount < 100) return `${props.connectionCount}`;
  else return "~";
});

const emit = defineEmits(["hover:start", "hover:end"]);

const diagramContext = useDiagramContext();
const { drawEdgeState } = diagramContext;

const isDeleted = computed(
  () => props.edge.def.changeStatus === "deleted" || props.edge.def.toDelete,
);
const isAdded = computed(() => props.edge.def.changeStatus === "added");

const willDeleteIfPendingEdgeCreated = computed(() => {
  return drawEdgeState.value.edgeKeysToDelete.includes(props.edge.uniqueKey);
});

const defaultStrokeColor = computed(() =>
  theme.value === "dark" ? COLOR_PALETTE.shade[0] : COLOR_PALETTE.shade[100],
);

const strokeColor = computed(() => {
  if (isDevMode && props.edge.def.isInferred) {
    return "rgba(100,50,255,0.1)";
  }

  if (isAdded.value) return COLOR_PALETTE.success[500];
  if (isDeleted.value) return COLOR_PALETTE.destructive[500];
  return defaultStrokeColor.value;
});

const points = computed(() => {
  if (!props.fromPoint || !props.toPoint) return;
  const fromPointWithGap = pointAlongLinePx(
    props.fromPoint,
    props.toPoint,
    SOCKET_SIZE / 2,
  );
  const toPointWithGap = pointAlongLinePx(
    props.toPoint,
    props.fromPoint,
    SOCKET_SIZE / 2,
  );
  const p = [
    fromPointWithGap.x,
    fromPointWithGap.y,
    toPointWithGap.x,
    toPointWithGap.y,
  ];
  if (p.some((_p) => Number.isNaN(_p))) return;
  return p;
});

const centerPoint = computed(() => {
  if (!props.fromPoint || !props.toPoint) return;
  return pointAlongLinePct(props.fromPoint, props.toPoint, 0.5);
});

const viewStore = useViewsStore();

const selectedComponentId = computed(() => viewStore.selectedComponent?.def.id);

const isFromOrToSelected = computed(
  () =>
    props.edge.def.fromComponentId === selectedComponentId.value ||
    props.edge.def.toComponentId === selectedComponentId.value,
);
const showEdge = computed(() => {
  if (props.edge.def.isInferred) {
    return false;
  }

  return isFromOrToSelected.value || props.isSelected;
});

function onMouseOver() {
  viewStore.setHoveredEdgeId(props.edge.def.id);
}

function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  viewStore.setHoveredEdgeId(null);
}

function onMouseDown(_e: KonvaEventObject<MouseEvent>) {
  // e.cancelBubble = true; // stops dragging of parent
}

// defineExpose({ recalculatePoints });
</script>
