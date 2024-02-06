<template>
  <v-group v-if="shouldDraw && points && centerPoint">
    <v-line
      :config="{
        visible: isHovered || isSelected,
        points,
        stroke: SELECTION_COLOR,
        strokeWidth: isSelected ? 9 : 5,
        listening: false,
      }"
    />
    <!-- <v-line
      :config="{
        points,
        stroke: '#000',
        strokeWidth: 4,
        listening: false,
        opacity: 0.4,
      }"
      @mouseover="onMouseOver"
      @mouseout="onMouseOut"
      @mousedown="onMouseDown"
    /> -->
    <v-line
      :config="{
        id: edge.uniqueKey,
        points,
        stroke: strokeColor,
        strokeWidth: 2,
        hitStrokeWidth: 10,
        listening: !edge.def.isInvisible,
        opacity: mainLineOpacity,
        dash: [10, 10],
        dashEnabled: isDeleted,
        shadowColor: '#000',
        shadowBlur: 1,
        shadowEnabled: isHovered || isSelected,
      }"
      @mouseover="onMouseOver"
      @mouseout="onMouseOut"
      @mousedown="onMouseDown"
    />

    <v-group
      v-if="
        !edge.def.isInvisible &&
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
          icon="scissors"
          :color="getToneColorHex('destructive')"
          :size="20"
        />
      </template>
      <template v-else-if="isAdded">
        <DiagramIcon
          icon="plus-square"
          :color="getToneColorHex('success')"
          :size="20"
          shadeBg
        />
      </template>
      <template v-else-if="isDeleted">
        <DiagramIcon
          icon="minus-square"
          shadeBg
          :color="getToneColorHex('destructive')"
          :size="20"
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
import { useComponentsStore } from "@/store/components.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { SELECTION_COLOR, SOCKET_SIZE } from "./diagram_constants";
import { DiagramEdgeData } from "./diagram_types";
import { pointAlongLinePct, pointAlongLinePx } from "./utils/math";
import DiagramIcon from "./DiagramIcon.vue";
import { useDiagramContext } from "./ModelingDiagram.vue";

const isDevMode = import.meta.env.DEV;

const props = defineProps({
  edge: {
    type: Object as PropType<DiagramEdgeData>,
    required: true,
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

const emit = defineEmits(["hover:start", "hover:end"]);

const { theme } = useTheme();

const featureFlagsStore = useFeatureFlagsStore();

const diagramContext = useDiagramContext();
const { drawEdgeState } = diagramContext;

const isDeleted = computed(() => props.edge.def.changeStatus === "deleted");
const isAdded = computed(() => props.edge.def.changeStatus === "added");

const willDeleteIfPendingEdgeCreated = computed(() => {
  return drawEdgeState.value.edgeKeysToDelete.includes(props.edge.uniqueKey);
});

const defaultStrokeColor = computed(() =>
  theme.value === "dark" ? COLOR_PALETTE.shade[0] : COLOR_PALETTE.shade[100],
);

const strokeColor = computed(() => {
  if (isDevMode && props.edge.def.isInvisible) {
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
  return [
    fromPointWithGap.x,
    fromPointWithGap.y,
    toPointWithGap.x,
    toPointWithGap.y,
  ];
});

const centerPoint = computed(() => {
  if (!props.fromPoint || !props.toPoint) return;
  return pointAlongLinePct(props.fromPoint, props.toPoint, 0.5);
});

const selectedNodeId = computed(
  () => componentsStore.selectedComponent?.nodeId,
);

const mainLineOpacity = computed(() => {
  if (willDeleteIfPendingEdgeCreated.value) return 0.3;
  if (isDeleted.value) return 0.75;

  if (featureFlagsStore.SHOW_EDGES_ON_SELECT) {
    if (
      props.edge.def.fromNodeId === selectedNodeId.value ||
      props.edge.def.toNodeId === selectedNodeId.value
    ) {
      return 0.8;
    } else {
      return 0.1;
    }
  } else {
    return 1;
  }
});

const componentsStore = useComponentsStore();

function onMouseOver() {
  componentsStore.setHoveredEdgeId(props.edge.def.id);
}

function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  componentsStore.setHoveredEdgeId(null);
}

function onMouseDown(_e: KonvaEventObject<MouseEvent>) {
  // e.cancelBubble = true; // stops dragging of parent
}

const shouldDraw = computed(() =>
  isDevMode ? true : !props.edge.def.isInvisible,
);

// defineExpose({ recalculatePoints });
</script>
