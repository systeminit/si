<template>
  <v-group
    ref="groupRef"
    :config="{
      id: node.uniqueKey,
      x: position.x,
      y: position.y,
    }"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
  >
    <!-- selection box outline -->
    <v-rect
      v-if="isHovered || isSelected"
      :config="{
        width: nodeWidth + 8,
        height: nodeHeight + 8,
        x: -halfWidth - 4,
        y: -4,
        cornerRadius: CORNER_RADIUS + 3,
        stroke: SELECTION_COLOR,
        strokeWidth: isSelected ? 3 : 1,
        listening: false,
      }"
    />

    <v-group :config="{ opacity: isDeleted ? 0.5 : 1 }">
      <!-- box background - also used by layout manager to figure out nodes location and size -->
      <v-rect
        :config="{
          id: `${node.uniqueKey}--bg`,
          width: nodeWidth,
          height: nodeHeight,
          x: -halfWidth,
          y: 0,
          cornerRadius: CORNER_RADIUS,
          fill: colors.bodyBg,
          fillAfterStrokeEnabled: true,
          stroke: colors.border,
          strokeWidth: 4,
          shadowColor: 'black',
          shadowBlur: 8,
          shadowOffset: { x: 3, y: 3 },
          shadowOpacity: 0.4,
          shadowEnabled: false,
        }"
      />

      <!-- package/type icon -->
      <DiagramIcon
        v-if="node.def.typeIcon"
        :icon="node.def.typeIcon"
        :color="colors.icon"
        :size="22"
        :x="-halfWidth + 5"
        :y="5"
        origin="top-left"
      />

      <!-- header text -->
      <v-text
        ref="titleTextRef"
        :config="{
          x: -halfWidth + 24 + 8,
          y: 4,
          verticalAlign: 'top',
          align: 'left',
          text: truncatedNodeTitle,
          width: nodeWidth - 24 - 24 - 8,
          padding: 0,
          fill: colors.headerText,
          fontStyle: 'bold',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
        }"
      />

      <v-text
        ref="subtitleTextRef"
        :config="{
          x: -halfWidth + 24 + 8,
          y: headerTextHeight + 6,
          verticalAlign: 'top',
          align: 'left',
          text: node.def.subtitle,
          width: nodeWidth - 24 - 24 - 8,
          padding: 0,
          fill: colors.bodyText,
          fontFamily: DIAGRAM_FONT_FAMILY,
          fontSize: 11,
          fontStyle: 'italic',
          listening: false,
        }"
      />

      <!-- header bottom border -->
      <v-line
        :config="{
          points: [-halfWidth, nodeHeaderHeight, halfWidth, nodeHeaderHeight],
          stroke: colors.border,
          strokeWidth: 1,
          listening: false,
          opacity: 0.7,
        }"
      />

      <!-- sockets -->
      <v-group
        :config="{
          x: -halfWidth,
          y: nodeHeaderHeight + subtitleTextHeight + SOCKET_MARGIN_TOP,
        }"
      >
        <DiagramNodeSocket
          v-for="(socket, i) in leftSockets"
          :key="socket.uniqueKey"
          :socket="socket"
          :y="i * SOCKET_GAP"
          :connectedEdges="connectedEdgesBySocketKey[socket.uniqueKey]"
          :drawEdgeState="drawEdgeState"
          :nodeWidth="nodeWidth"
          @hover:start="onSocketHoverStart(socket)"
          @hover:end="onSocketHoverEnd(socket)"
        />
      </v-group>

      <v-group
        :config="{
          x: halfWidth,
          y:
            nodeHeaderHeight +
            SOCKET_MARGIN_TOP +
            subtitleTextHeight +
            SOCKET_GAP * leftSockets.length,
        }"
      >
        <DiagramNodeSocket
          v-for="(socket, i) in rightSockets"
          :key="socket.uniqueKey"
          :socket="socket"
          :y="i * SOCKET_GAP"
          :connectedEdges="connectedEdgesBySocketKey[socket.uniqueKey]"
          :drawEdgeState="drawEdgeState"
          :nodeWidth="nodeWidth"
          @hover:start="onSocketHoverStart(socket)"
          @hover:end="onSocketHoverEnd(socket)"
        />
      </v-group>

      <!-- status icons -->
      <v-group
        v-if="node.def.statusIcons?.length"
        :config="{
          x: halfWidth - node.def.statusIcons.length * 36 + 18,
          y:
            nodeHeaderHeight +
            subtitleTextHeight +
            SOCKET_MARGIN_TOP +
            SOCKET_GAP * (leftSockets.length + rightSockets.length),
        }"
      >
        <template
          v-for="(statusIcon, i) in node.def.statusIcons"
          :key="`status-icon-${i}`"
        >
          <DiagramIcon
            :icon="statusIcon.icon"
            :color="statusIcon.color || diagramConfig?.toneColors?.[statusIcon.tone!] || diagramConfig?.toneColors?.neutral || '#AAA'"
            :size="20"
            :x="i * 30"
            :y="0"
            origin="top-left"
          />

          <v-line
            v-if="i !== node.def.statusIcons.length - 1"
            :config="{
              points: [i * 36 + 24, 0, i * 36 + 24, 18],
              stroke: '#777',
              strokeWidth: 1,
              listening: false,
              opacity: 0.7,
            }"
          />
        </template>
      </v-group>

      <!--  spinner overlay  -->
      <v-group
        ref="overlay"
        :config="{
          x: -halfWidth,
          y: nodeHeaderHeight,
          opacity: 0,
          listening: false,
        }"
      >
        <!--  transparent overlay  -->
        <v-rect
          :config="{
            width: nodeWidth,
            height: nodeBodyHeight,
            x: 0,
            y: 0,
            cornerRadius: [0, 0, CORNER_RADIUS, CORNER_RADIUS],
            fill: 'rgba(255,255,255,0.70)',
          }"
        />
        <DiagramIcon
          icon="loader"
          :color="diagramConfig?.toneColors?.info || '#AAA'"
          :size="overlayIconSize"
          :x="halfWidth"
          :y="nodeBodyHeight / 2"
        />
      </v-group>
    </v-group>

    <!-- change status indicators -->
    <!-- deleted X overlay (large centered) -->
    <DiagramIcon
      v-if="isDeleted"
      :icon="deleteIcon"
      :color="diagramConfig?.toneColors?.destructive"
      :size="DELETED_X_SIZE"
      :x="0"
      :y="nodeHeight / 2"
    />

    <!-- added/modified indicator (smaller, bottom left) -->
    <DiagramIcon
      v-if="isAdded || isModified"
      :icon="isAdded ? 'plus' : 'tilde'"
      :bgColor="
        isAdded
          ? diagramConfig?.toneColors?.success
          : diagramConfig?.toneColors?.warning
      "
      circleBg
      :color="theme === 'dark' ? '#000' : '#FFF'"
      :size="20"
      :x="halfWidth - 5 - 10"
      :y="nodeHeaderHeight / 2"
      origin="center"
    />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, nextTick, PropType, ref, watch } from "vue";
import * as _ from "lodash-es";
import tinycolor from "tinycolor2";

import { KonvaEventObject } from "konva/lib/Node";
import { Tween } from "konva/lib/Tween";
import { Vector2d } from "konva/lib/types";
import { IconNames, useTheme } from "@si/vue-lib/design-system";
import {
  DiagramDrawEdgeState,
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramNodeData,
  DiagramSocketData,
  ElementHoverMeta,
} from "./diagram_types";
import DiagramNodeSocket from "./DiagramNodeSocket.vue";

import {
  SOCKET_GAP,
  CORNER_RADIUS,
  NODE_PADDING_BOTTOM,
  SOCKET_MARGIN_TOP,
  DEFAULT_NODE_COLOR,
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
  SOCKET_SIZE,
  NODE_WIDTH,
} from "./diagram_constants";
import DiagramIcon from "./DiagramIcon.vue";
import { useDiagramConfig } from "./utils/use-diagram-context-provider";

const props = defineProps({
  node: {
    type: Object as PropType<DiagramNodeData>,
    required: true,
  },
  tempPosition: {
    type: Object as PropType<Vector2d>,
  },
  connectedEdges: {
    type: Object as PropType<DiagramEdgeData[]>,
    default: () => ({}),
  },
  drawEdgeState: {
    type: Object as PropType<DiagramDrawEdgeState>,
    default: () => ({}),
  },
  isHovered: Boolean,
  isSelected: Boolean,
  deleteIcon: { type: String as PropType<IconNames>, default: "x" },
});

const emit = defineEmits<{
  (e: "hover:start", meta?: ElementHoverMeta): void;
  (e: "hover:end"): void;
  (e: "resize"): void;
}>();

const { theme } = useTheme();
const diagramConfig = useDiagramConfig();

const isDeleted = computed(() => props.node.def.changeStatus === "deleted");
const isModified = computed(() => props.node.def.changeStatus === "modified");
const isAdded = computed(() => props.node.def.changeStatus === "added");

const DELETED_X_SIZE = 100;

// template refs
const titleTextRef = ref();
const subtitleTextRef = ref();
const groupRef = ref();

const leftSockets = computed(() =>
  _.filter(
    props.node.sockets,
    (s) => s.def.nodeSide === "left" && s.def.label !== "Frame",
  ),
);
const rightSockets = computed(() =>
  _.filter(
    props.node.sockets,
    (s) => s.def.nodeSide === "right" && s.def.label !== "Frame",
  ),
);

const connectedEdgesBySocketKey = computed(() => {
  const lookup: Record<DiagramElementUniqueKey, DiagramEdgeData[]> = {};
  _.each(props.connectedEdges, (edge) => {
    lookup[edge.fromSocketKey] ||= [];
    lookup[edge.fromSocketKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
    lookup[edge.toSocketKey] ||= [];
    lookup[edge.toSocketKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
  });
  return lookup;
});

const MAX_TITLE_LENGTH = 80;

const truncatedNodeTitle = computed(() => {
  if (props.node.def.title.length > MAX_TITLE_LENGTH) {
    return `${props.node.def.title.substring(0, MAX_TITLE_LENGTH)}...`;
  } else return props.node.def.title;
});

const nodeWidth = computed(() => NODE_WIDTH);
const halfWidth = computed(() => nodeWidth.value / 2);

const overlayIconSize = computed(() => nodeWidth.value / 3);

const headerTextHeight = ref(20);
const subtitleTextHeight = ref(0);
watch(
  [nodeWidth, () => props.node.def.title, () => props.node.def.subtitle],
  () => {
    // we have to let the new header be drawn on the canvas before we can check the height
    nextTick(recalcHeaderHeight);
  },
  { immediate: true },
);

function recalcHeaderHeight() {
  headerTextHeight.value =
    titleTextRef.value?.getNode()?.getSelfRect().height || 20;
  subtitleTextHeight.value = props.node.def.subtitle
    ? subtitleTextRef.value?.getNode()?.getSelfRect().height
    : 10;
}

const nodeHeaderHeight = computed(
  () => headerTextHeight.value + subtitleTextHeight.value + 6 + 4,
);
const nodeBodyHeight = computed(() => {
  return (
    subtitleTextHeight.value +
    SOCKET_MARGIN_TOP +
    SOCKET_GAP * (leftSockets.value.length + rightSockets.value.length - 1) +
    SOCKET_SIZE / 2 +
    // TODO: this isnt right yet!
    NODE_PADDING_BOTTOM +
    (props.node.def.statusIcons?.length ? 30 : 0)
  );
});
const nodeHeight = computed(
  () => nodeHeaderHeight.value + nodeBodyHeight.value,
);

const position = computed(() => props.tempPosition || props.node.def.position);

watch([nodeWidth, nodeHeight, position], () => {
  // we call on nextTick to let the component actually update itself on the stage first
  // because parent responds to this event by finding shapes on the stage and looking at location/dimensions
  nextTick(() => emit("resize"));
});

const colors = computed(() => {
  const primaryColor = tinycolor(props.node.def.color || DEFAULT_NODE_COLOR);

  // body bg
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  const bodyBg = tinycolor(bodyBgHsl);

  const bodyText = theme.value === "dark" ? "#FFF" : "#000";
  return {
    border: primaryColor.toRgbString(),
    icon: bodyText,
    headerText: bodyText,
    bodyBg: bodyBg.toRgbString(),
    bodyText,
  };
});

const overlay = ref();
watch([() => props.node.def.isLoading, overlay], () => {
  const node = overlay.value?.getNode();
  if (!node) return;
  const transition = new Tween({
    node,
    duration: 0.1,
    opacity: props.node.def.isLoading ? 1 : 0,
  });
  transition.play();
});

function onMouseOver(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:start");
}

function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:end");
}

function onSocketHoverStart(socket: DiagramSocketData) {
  emit("hover:start", { type: "socket", socket });
}

function onSocketHoverEnd(_socket: DiagramSocketData) {
  emit("hover:end");
}
</script>
