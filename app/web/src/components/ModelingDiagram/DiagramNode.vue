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
    <v-group :config="{ opacity: isDeleted ? 0.5 : 1 }">
      <!-- drop shadow -->
      <!-- <v-rect
        :config="{
          width: nodeWidth,
          height: nodeHeight,
          x: -halfWidth - 10,
          y: 10,
          cornerRadius: CORNER_RADIUS,
          fill: colors.bodyBg,
          fillAfterStrokeEnabled: true,
        }"
      /> -->

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
          stroke: colors.border,
          strokeWidth: 2,
          hitStrokeWidth: 0,
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

      <!-- parent frame attachment indicator -->
      <DiagramIcon
        v-if="parentComponentId"
        icon="frame"
        :size="16"
        :x="-halfWidth + 12"
        :y="nodeHeaderHeight + nodeBodyHeight - 12"
        :color="colors.parentColor"
        @mouseover="(e) => onMouseOver(e, 'parent')"
        @mouseout="onMouseOut"
      />

      <!-- header bottom border -->
      <v-line
        :config="{
          points: [-halfWidth, nodeHeaderHeight, halfWidth, nodeHeaderHeight],
          stroke: colors.border,
          strokeWidth: 1,
          listening: false,
          opacity: 1,
        }"
      />

      <!-- status icons -->
      <v-group
        :config="{
          x: halfWidth - 2,
          y: nodeHeight - 2,
        }"
      >
        <DiagramIcon
          v-for="(statusIcon, i) in _.reverse(_.slice(node.def.statusIcons))"
          :key="`status-icon-${i}`"
          :icon="statusIcon.icon"
          :color="statusIcon.color || statusIcon.tone
            ? getToneColorHex(statusIcon.tone!)
            : getToneColorHex('neutral')"
          :size="24 + (statusIconHovers[i] ? 4 : 0)"
          :x="i * -26 + (statusIconHovers[i] ? 2 : 0)"
          :y="statusIconHovers[i] ? 2 : 0"
          origin="bottom-right"
          @click="statusIcon.tabSlug ? onClick(statusIcon.tabSlug) : _.noop"
          @mouseover="statusIconHovers[i] = true"
          @mouseout="statusIconHovers[i] = false"
        />
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
          :color="getToneColorHex('info')"
          :size="overlayIconSize"
          :x="halfWidth"
          :y="nodeBodyHeight / 2"
        />
      </v-group>

      <!-- added/modified indicator -->
      <DiagramIcon
        v-if="isAdded || isModified"
        :icon="isAdded ? 'plus-square' : 'tilde-square'"
        :color="
          isAdded ? getToneColorHex('success') : getToneColorHex('warning')
        "
        :size="24 + (diffIconHover ? 4 : 0)"
        :x="halfWidth - 2 - 12"
        :y="nodeHeaderHeight / 2"
        origin="center"
        @click="onClick('diff')"
        @mouseover="diffIconHover = true"
        @mouseout="diffIconHover = false"
      />

      <!-- added/modified icon hover -->
      <!-- <v-rect
        v-if="diffIconHover && (isAdded || isModified)"
        :config="{
          width: 24,
          height: 24,
          x: halfWidth - 2 - 24,
          y: nodeHeaderHeight / 2 - 12,
          cornerRadius: CORNER_RADIUS + 3,
          stroke: SELECTION_COLOR,
          strokeWidth: 2,
          listening: false,
        }"
      /> -->
    </v-group>

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

    <!-- sockets -->
    <v-group :config="{ opacity: isDeleted ? 0.5 : 1 }">
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
          :nodeWidth="nodeWidth"
          @hover:start="onSocketHoverStart(socket)"
          @hover:end="onSocketHoverEnd(socket)"
        />
      </v-group>
    </v-group>

    <!-- change status indicators -->
    <!-- deleted icon overlay (large centered) -->
    <DiagramIcon
      v-if="isDeleted"
      icon="minus-square"
      shadeBg
      :color="getToneColorHex('destructive')"
      :size="DELETED_X_SIZE"
      :x="0"
      :y="nodeHeight / 2"
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
import { getToneColorHex, useTheme } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import {
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramNodeData,
  DiagramSocketData,
} from "./diagram_types";
import DiagramNodeSocket from "./DiagramNodeSocket.vue";

import {
  CORNER_RADIUS,
  DEFAULT_NODE_COLOR,
  DIAGRAM_FONT_FAMILY,
  NODE_PADDING_BOTTOM,
  NODE_WIDTH,
  SELECTION_COLOR,
  SOCKET_GAP,
  SOCKET_MARGIN_TOP,
  SOCKET_SIZE,
} from "./diagram_constants";
import DiagramIcon from "./DiagramIcon.vue";
import { useDiagramContext } from "./ModelingDiagram.vue";

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
  isHovered: Boolean,
  isSelected: Boolean,
});

const emit = defineEmits<{
  (e: "resize"): void;
}>();

const componentsStore = useComponentsStore();
const componentId = computed(() => props.node.def.componentId);

const diffIconHover = ref(false);
const statusIconHovers = ref(
  new Array(props.node.def.statusIcons?.length || 0).fill(false),
);

const { theme } = useTheme();

const diagramContext = useDiagramContext();
const { edgeDisplayMode } = diagramContext;

const isDeleted = computed(() => props.node.def.changeStatus === "deleted");
const isModified = computed(() => props.node.def.changeStatus === "modified");
const isAdded = computed(() => props.node.def.changeStatus === "added");

const DELETED_X_SIZE = 80;

// template refs
const titleTextRef = ref();
const subtitleTextRef = ref();
const groupRef = ref();

const actualSockets = computed(() =>
  _.filter(props.node.sockets, (s) => {
    const should_skip = s.def.label === "Frame";

    return !should_skip;
  }),
);

const leftSockets = computed(() => {
  const leftSockets = _.filter(
    actualSockets.value,
    (s) => s.def.nodeSide === "left",
  );

  return _.sortBy(leftSockets, (s) => s.def.label);
});
const rightSockets = computed(() => {
  const rightSockets = _.filter(
    actualSockets.value,
    (s) => s.def.nodeSide === "right",
  );

  return _.sortBy(rightSockets, (s) => s.def.label);
});

const connectedEdgesBySocketKey = computed(() => {
  const lookup: Record<DiagramElementUniqueKey, DiagramEdgeData[]> = {};
  _.each(props.connectedEdges, (edge) => {
    lookup[edge.fromExternalProviderKey] ||= [];
    lookup[edge.fromExternalProviderKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
    lookup[edge.toExplicitInternalProviderKey] ||= [];
    lookup[edge.toExplicitInternalProviderKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
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

const parentComponentId = computed(() => _.last(props.node.def.ancestorIds));

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

  if (edgeDisplayMode.value === "EDGES_UNDER") bodyBg.setAlpha(0.5);

  const bodyText = theme.value === "dark" ? "#FFF" : "#000";
  return {
    border: primaryColor.toRgbString(),
    icon: bodyText,
    headerText: bodyText,
    bodyBg: bodyBg.toRgbString(),
    bodyText,
    parentColor:
      componentsStore.componentsById[parentComponentId.value || ""]?.color ||
      "#FFF",
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

function onMouseOver(evt: KonvaEventObject<MouseEvent>, type?: "parent") {
  evt.cancelBubble = true;
  componentsStore.setHoveredComponentId(
    componentId.value,
    type ? { type } : undefined,
  );
}

function onMouseOut() {
  componentsStore.setHoveredComponentId(null);
}

function onSocketHoverStart(socket: DiagramSocketData) {
  componentsStore.setHoveredComponentId(componentId.value, {
    type: "socket",
    socket,
  });
}

function onSocketHoverEnd(_socket: DiagramSocketData) {
  componentsStore.setHoveredComponentId(null);
}

function onClick(detailsTabSlug: string) {
  componentsStore.setSelectedComponentId(props.node.def.id, {
    detailsTab: detailsTabSlug,
  });
}
</script>
