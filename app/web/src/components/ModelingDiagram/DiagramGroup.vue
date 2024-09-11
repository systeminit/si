<template>
  <v-group
    ref="groupRef"
    :config="{
      id: group.uniqueKey,
      x: position.x,
      y: position.y,
      ...(isDeleted && { opacity: 0.5 }),
    }"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
  >
    <!-- selection box outline -->
    <v-rect
      v-if="isHovered || highlightParent || highlightAsNewParent"
      :config="{
        width: nodeWidth + 8,
        height: nodeHeight + 8,
        x: -halfWidth - 4,
        y: -4 - nodeHeaderHeight - GROUP_HEADER_BOTTOM_MARGIN,
        cornerRadius: CORNER_RADIUS + 3,
        stroke: SELECTION_COLOR,
        strokeWidth: 1,
        listening: false,
      }"
    />
    <!-- box background - also used by layout manager to figure out nodes location and size -->
    <!-- <v-rect
      :config="{
        id: `${group.uniqueKey}--bg`,
        width: nodeWidth,
        height: nodeHeight,
        x: -halfWidth,
        y: 0,
      }"
    /> -->

    <v-rect
      :config="{
        id: `${group.uniqueKey}--bg`,
        width: nodeWidth,
        height: nodeBodyHeight,
        x: -halfWidth,
        y: 0,
        cornerRadius: CORNER_RADIUS,
        fill: colors.bodyBg,
        fillAfterStrokeEnabled: true,
        stroke: colors.headerBg,
        strokeWidth: 3,
        dash: [8, 8],
      }"
    />

    <!--  Node Body  -->
    <v-rect
      :config="{
        id: `${group.uniqueKey}--bg`,
        width: nodeWidth,
        height: nodeBodyHeight,
        x: -halfWidth,
        y: 0,
        cornerRadius: CORNER_RADIUS,
        fill: colors.bodyBg,
        fillAfterStrokeEnabled: true,
        stroke: colors.headerBg,
        strokeWidth: 3,
        dash: [8, 8],

        shadowForStrokeEnabled: false,
        hitStrokeWidth: 0,
        shadowColor: 'black',
        shadowBlur: 3,
        shadowOffset: { x: 8, y: 8 },
        shadowOpacity: 0.3,
        shadowEnabled: !parentComponentId,
      }"
    />

    <!-- resize handles -->
    <template v-if="!collapsed">
      <!--  left side handle  -->
      <v-line
        :config="{
          points: [
            -nodeWidth / 2,
            -(nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN),
            -nodeWidth / 2,
            nodeBodyHeight,
          ],
          hitStrokeWidth: GROUP_RESIZE_HANDLE_SIZE,
        }"
        @mouseover="onResizeHover('left', $event)"
        @mouseout="onMouseOut"
      />
      <!-- right side handle   -->
      <v-line
        :config="{
          points: [
            nodeWidth / 2,
            -(nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN),
            nodeWidth / 2,
            nodeBodyHeight,
          ],
          hitStrokeWidth: GROUP_RESIZE_HANDLE_SIZE,
        }"
        @mouseover="onResizeHover('right', $event)"
        @mouseout="onMouseOut"
      />
      <!-- Bottom Handle -->
      <v-line
        :config="{
          points: [
            -nodeWidth / 2,
            nodeBodyHeight,
            nodeWidth / 2,
            nodeBodyHeight,
          ],
          hitStrokeWidth: GROUP_RESIZE_HANDLE_SIZE,
        }"
        @mouseover="onResizeHover('bottom', $event)"
        @mouseout="onMouseOut"
      />
      <!-- Bottom Left Handle -->
      <v-circle
        :config="{
          width: GROUP_RESIZE_HANDLE_SIZE,
          height: GROUP_RESIZE_HANDLE_SIZE,
          x: -nodeWidth / 2,
          y: nodeBodyHeight,
        }"
        @mouseover="onResizeHover('bottom-left', $event)"
        @mouseout="onMouseOut"
      />
      <!-- Bottom Right Handle -->
      <v-circle
        :config="{
          width: GROUP_RESIZE_HANDLE_SIZE,
          height: GROUP_RESIZE_HANDLE_SIZE,
          x: nodeWidth / 2,
          y: nodeBodyHeight,
        }"
        @mouseover="onResizeHover('bottom-right', $event)"
        @mouseout="onMouseOut"
      />
      <!-- Top Handle -->
      <v-line
        :config="{
          points: [
            -nodeWidth / 2,
            -(nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN),
            nodeWidth / 2,
            -(nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN),
          ],
          hitStrokeWidth: GROUP_RESIZE_HANDLE_SIZE,
        }"
        @mouseover="onResizeHover('top', $event)"
        @mouseout="onMouseOut"
      />
      <!-- Top Left Handle -->
      <v-circle
        :config="{
          width: GROUP_RESIZE_HANDLE_SIZE,
          height: GROUP_RESIZE_HANDLE_SIZE,
          x: -nodeWidth / 2,
          y: -(nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN),
        }"
        @mouseover="onResizeHover('top-left', $event)"
        @mouseout="onMouseOut"
      />
      <!-- Top Right Handle -->
      <v-circle
        :config="{
          width: GROUP_RESIZE_HANDLE_SIZE,
          height: GROUP_RESIZE_HANDLE_SIZE,
          x: nodeWidth / 2,
          y: -(nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN),
        }"
        @mouseover="onResizeHover('top-right', $event)"
        @mouseout="onMouseOut"
      />
    </template>

    <!-- sockets -->
    <v-group
      v-if="!collapsed"
      :config="{
        x: -halfWidth - 1,
        y: nodeHeaderHeight + SOCKET_MARGIN_TOP,
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
      v-if="!collapsed"
      :config="{
        x: halfWidth + 1,
        y:
          nodeHeaderHeight +
          SOCKET_MARGIN_TOP +
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

    <!-- header -->
    <v-group
      :config="{
        x: -halfWidth,
        y: -nodeHeaderHeight - GROUP_HEADER_BOTTOM_MARGIN,
      }"
    >
      <!-- header background -->
      <!--  TODO check with mark what this width should be   -->
      <v-rect
        :config="{
          cornerRadius: CORNER_RADIUS,
          fill: colors.headerBg,
          x: 0,
          y: 0,
          width: headerWidth,
          height: headerTextHeight,
        }"
      />

      <DiagramIcon
        icon="chevron"
        origin="top-left"
        :rotation="collapsed ? 90 : 180"
        :size="40"
        :x="38"
        :y="collapsed ? 2 : 42"
        :color="colors.headerText"
        @click="toggleChevron"
      />

      <DiagramIcon
        :icon="COMPONENT_TYPE_ICONS[group.def.componentType]"
        origin="top-left"
        :size="32"
        :x="32"
        :y="5"
        :color="colors.headerText"
      />

      <!-- header text -->

      <!-- rename hitbox -->
      <v-rect
        v-if="!collapsed"
        :config="{
          ...renameHitbox,
          ...(debug && { fill: 'red' }),
        }"
        @mouseout="mouseOutRename"
        @mouseover="mouseOverRename"
        @click="renameIfSelected"
        @dblclick="rename"
      />

      <!-- component name -->
      <v-text
        ref="titleTextRef"
        :config="{
          x: 30 + GROUP_HEADER_ICON_SIZE - 2,
          y: 2,
          verticalAlign: 'top',
          align: 'left',
          width: headerWidth - GROUP_HEADER_ICON_SIZE * 2,
          text: group.def.title,
          padding: 6,
          fill: colors.headerText,
          fontSize: GROUP_TITLE_FONT_SIZE,
          fontStyle: 'bold',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
          wrap: 'none',
          ellipsis: true,
        }"
      />

      <!-- component type and child count -->
      <v-text
        ref="subtitleTextRef"
        :config="{
          x: 30 + GROUP_HEADER_ICON_SIZE - 2,
          y: 20,
          verticalAlign: 'top',
          align: 'left',
          width: headerWidth - GROUP_HEADER_ICON_SIZE * 2,
          text: `${group.def.subtitle}: ${childCount ?? 0}`,
          padding: 6,
          fill: colors.headerText,
          fontSize: GROUP_TITLE_FONT_SIZE,
          fontStyle: 'italic',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
          wrap: 'none',
          ellipsis: true,
        }"
      />
    </v-group>

    <!-- parent frame attachment indicator -->
    <DiagramIcon
      v-if="parentComponentId"
      icon="frame"
      :size="16"
      :x="-halfWidth + 12"
      :y="nodeBodyHeight - 12"
      :color="colors.parentColor"
      @mouseover="(e) => onMouseOver(e, 'parent')"
      @mouseout="onMouseOut"
    />

    <!-- status icons for collapsed children-->
    <v-group
      v-if="collapsed"
      :config="{
        x: halfWidth - 2,
        y: 0,
      }"
    >
      <v-text
        :config="{
          x: -58,
          y: nodeBodyHeight - 25,
          verticalAlign: 'top',
          align: 'right',
          width: 32,
          text: `${group.def.numChildrenResources}`,
          fill: colors.labelText,
          fontSize: 16,
          fontStyle: 'bold',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
          wrap: 'none',
          ellipsis: false,
        }"
      ></v-text>
      <DiagramIcon
        icon="check-hex"
        :color="getToneColorHex('success')"
        :size="24"
        :x="0"
        :y="nodeBodyHeight - 5"
        origin="bottom-right"
      />
      <v-text
        :config="{
          x: -112,
          y: nodeBodyHeight - 25,
          verticalAlign: 'top',
          align: 'right',
          width: 32,
          text: `${group.def.numChildren}`,
          fill: colors.labelText,
          fontSize: 16,
          fontStyle: 'bold',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
          wrap: 'none',
          ellipsis: false,
        }"
      />
      <DiagramIcon
        icon="check-hex-outline"
        :color="getToneColorHex('success')"
        :size="24"
        :x="-55"
        :y="nodeBodyHeight - 5"
        origin="bottom-right"
      />
    </v-group>

    <!-- status icons -->
    <v-group
      v-if="statusIcons?.length && !collapsed"
      :config="{
        x: halfWidth - 2,
        y: 0,
      }"
    >
      <DiagramIcon
        v-for="(statusIcon, i) in _.reverse(_.slice(statusIcons))"
        :key="`status-icon-${i}`"
        :icon="statusIcon.icon"
        :color="
          statusIcon.color || statusIcon.tone
            ? getToneColorHex(statusIcon.tone!)
            : getToneColorHex('neutral')
        "
        :size="24 + (statusIconHovers[i] ? 6 : 0)"
        :x="i * -26 + (statusIconHovers[i] ? 3 : 0)"
        :y="nodeBodyHeight - 5 + (statusIconHovers[i] ? 3 : 0)"
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
        y: 0,
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
          fill: 'rgba(255,255,255,0.30)',
        }"
      />
    </v-group>

    <DiagramIcon
      v-if="group.def.canBeUpgraded"
      :color="getToneColorHex('action')"
      :size="24 + (diffIconHover ? 4 : 0)"
      :x="halfWidth - GROUP_HEADER_ICON_SIZE - 36 / 2"
      :y="
        -nodeHeaderHeight +
        GROUP_HEADER_ICON_SIZE / 2 -
        GROUP_HEADER_BOTTOM_MARGIN +
        (nodeHeaderHeight - GROUP_HEADER_ICON_SIZE) / 2
      "
      icon="bolt"
      origin="center"
    />

    <!-- added/modified indicator -->
    <DiagramIcon
      v-if="isAdded || isModified || isDeleted"
      :icon="topRightIcon"
      :color="topRightIconColor"
      shadeBg
      :size="GROUP_HEADER_ICON_SIZE + (diffIconHover ? 8 : 0)"
      :x="halfWidth - GROUP_HEADER_ICON_SIZE / 2"
      :y="
        -nodeHeaderHeight +
        GROUP_HEADER_ICON_SIZE / 2 -
        GROUP_HEADER_BOTTOM_MARGIN +
        (nodeHeaderHeight - GROUP_HEADER_ICON_SIZE) / 2
      "
      origin="center"
      @click="onClick('diff')"
      @mouseover="diffIconHover = true"
      @mouseout="diffIconHover = false"
    />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, nextTick, onUpdated, PropType, ref, watch } from "vue";
import * as _ from "lodash-es";
import tinycolor from "tinycolor2";

import { KonvaEventObject } from "konva/lib/Node";
import {
  getToneColorHex,
  useTheme,
  COMPONENT_TYPE_ICONS,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import DiagramNodeSocket from "@/components/ModelingDiagram/DiagramNodeSocket.vue";
import {
  CORNER_RADIUS,
  DEFAULT_NODE_COLOR,
  DIAGRAM_FONT_FAMILY,
  GROUP_HEADER_BOTTOM_MARGIN,
  GROUP_HEADER_ICON_SIZE,
  GROUP_RESIZE_HANDLE_SIZE,
  GROUP_TITLE_FONT_SIZE,
  MIN_NODE_DIMENSION,
  SELECTION_COLOR,
  SOCKET_GAP,
  SOCKET_MARGIN_TOP,
} from "@/components/ModelingDiagram/diagram_constants";
import { trackEvent } from "@/utils/tracking";
import {
  QualificationStatus,
  statusIconsForComponent,
} from "@/store/qualifications.store";
import {
  DiagramDrawEdgeState,
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramGroupData,
  DiagramSocketData,
  ElementHoverMeta,
  SideAndCornerIdentifiers,
} from "./diagram_types";
import { useDiagramContext } from "./ModelingDiagram.vue";
import DiagramIcon from "./DiagramIcon.vue";

const props = defineProps({
  group: {
    type: Object as PropType<DiagramGroupData>,
    required: true,
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
  qualificationStatus: {
    type: String as PropType<QualificationStatus>,
    required: false,
  },
  collapsed: Boolean,
  debug: Boolean,
});

const diagramContext = useDiagramContext();

const componentId = computed(() => props.group.def.componentId);
const parentComponentId = computed(() => props.group.def.parentId);

const statusIcons = computed(() =>
  statusIconsForComponent(
    props.qualificationStatus,
    props.group.def.hasResource,
  ),
);

const diffIconHover = ref(false);
const statusIconHovers = ref(
  new Array(statusIcons?.value.length || 0).fill(false),
);

const emit = defineEmits<{
  (e: "resize"): void;
  (e: "rename", v: () => void): void;
}>();

const { theme } = useTheme();

const titleTextRef = ref();
const subtitleTextRef = ref();
const groupRef = ref();

const componentsStore = useComponentsStore();

const size = computed(
  () =>
    componentsStore.combinedElementSizes[props.group.uniqueKey] ||
    props.group.def.size || { width: 500, height: 500 },
);

const isDeleted = computed(
  () =>
    props.group.def.changeStatus === "deleted" ||
    props.group.def.toDelete ||
    props.group.def.fromBaseChangeSet,
);
const isModified = computed(() => props.group.def.changeStatus === "modified");
const isAdded = computed(() => props.group.def.changeStatus === "added");

const topRightIcon = computed(() => {
  if (isDeleted.value) return "minus-square";
  else if (isAdded.value) return "plus-square";
  else return "tilde-square";
});
const topRightIconColor = computed(() => {
  if (isDeleted.value) return getToneColorHex("destructive");
  else if (isAdded.value) return getToneColorHex("success");
  else return getToneColorHex("warning");
});

const childCount = computed(() => {
  const mappedChildren = _.map(
    props.group.def.childIds,
    (child) => componentsStore.componentsById[child],
  );

  const undeletedChildren = _.filter(mappedChildren, (child) =>
    _.isNil(child?.deletedInfo),
  );

  return undeletedChildren.length;
});

const nodeWidth = computed(() =>
  props.collapsed
    ? size.value.width
    : Math.max(size.value.width, MIN_NODE_DIMENSION),
);
const halfWidth = computed(() => nodeWidth.value / 2);
const headerWidth = computed(() =>
  !props.group.def.changeStatus || props.group.def.changeStatus === "unmodified"
    ? nodeWidth.value
    : nodeWidth.value - GROUP_HEADER_ICON_SIZE - 4,
);

const actualSockets = computed(() =>
  _.filter(props.group.sockets, (s) => {
    const should_skip = s.def.label === "Frame";

    return !should_skip;
  }),
);

const leftSockets = computed(() =>
  _.filter(actualSockets.value, (s) => s.def.nodeSide === "left"),
);
const rightSockets = computed(() =>
  _.filter(actualSockets.value, (s) => s.def.nodeSide === "right"),
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

const headerTextHeight = ref(20);
watch(
  [nodeWidth, () => props.group.def.title, () => props.group.def.subtitle],
  () => {
    // we have to let the new header be drawn on the canvas before we can check the height
    nextTick(recalcHeaderHeight);
  },
  { immediate: true },
);

function recalcHeaderHeight() {
  headerTextHeight.value =
    titleTextRef.value?.getNode()?.getSelfRect().height || 20;
  headerTextHeight.value *= 1.7;
}

const nodeHeaderHeight = computed(() => headerTextHeight.value);
const nodeBodyHeight = computed(() =>
  !props.collapsed
    ? Math.max(size.value.height, MIN_NODE_DIMENSION)
    : size.value.height,
);
const nodeHeight = computed(
  () =>
    nodeHeaderHeight.value + GROUP_HEADER_BOTTOM_MARGIN + nodeBodyHeight.value,
);

const position = computed(
  () =>
    componentsStore.combinedElementPositions[props.group.uniqueKey] ||
    props.group.def.position,
);

watch([nodeWidth, nodeHeight, position, actualSockets], () => {
  // we call on nextTick to let the component actually update itself on the stage first
  // because parent responds to this event by finding shapes on the stage and looking at location/dimensions
  nextTick(() => emit("resize"));
});

const colors = computed(() => {
  const primaryColor = tinycolor(props.group.def.color || DEFAULT_NODE_COLOR);

  // body bg
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  const bodyBg = tinycolor(bodyBgHsl);

  const bodyText = theme.value === "dark" ? "#FFF" : "#000";
  let headerText;
  if (primaryColor.toHsl().l < 0.5) {
    headerText = "#FFF";
  } else {
    headerText = "#000";
  }
  return {
    headerBg: primaryColor.toRgbString(),
    icon: "#000",
    headerText,
    bodyBg: bodyBg.toRgbString(),
    labelText: theme.value === "dark" ? "#FFF" : "#000",
    bodyText,
    parentColor:
      componentsStore.componentsById[parentComponentId.value || ""]?.color ||
      "#FFF",
  };
});

function onMouseOver(evt: KonvaEventObject<MouseEvent>, type?: string) {
  evt.cancelBubble = true;
  componentsStore.setHoveredComponentId(
    componentId.value,
    type ? ({ type } as ElementHoverMeta) : undefined,
  );
}

function onResizeHover(
  direction: SideAndCornerIdentifiers,
  evt: KonvaEventObject<MouseEvent>,
) {
  evt.cancelBubble = true;
  componentsStore.setHoveredComponentId(componentId.value, {
    type: "resize",
    direction,
  });
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

function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  componentsStore.setHoveredComponentId(null);
}

function onClick(detailsTabSlug: string) {
  componentsStore.setSelectedComponentId(componentId.value, {
    detailsTab: detailsTabSlug,
  });
}

const highlightParent = computed(() => {
  if (!componentsStore.hoveredComponent) return false;
  if (componentsStore.hoveredComponentMeta?.type !== "parent") return false;
  return (
    componentsStore.hoveredComponent.ancestorIds?.includes(componentId.value) ||
    false
  );
});

const highlightAsNewParent = computed(() => {
  return (
    diagramContext.moveElementsState.value.active &&
    diagramContext.moveElementsState.value.intoNewParentKey ===
      props.group.uniqueKey
  );
});

const component = computed(
  () => componentsStore.rawComponentsById[props.group.def.id],
);

const toggleChevron = () => {
  if (componentsStore.collapsedComponents.has(props.group.uniqueKey)) {
    componentsStore.expandComponents(props.group.uniqueKey);
    trackEvent("expand-components", {
      source: "diagram-group",
      schemaVariantName: component.value?.schemaVariantName,
      schemaName: component.value?.schemaName,
      hasParent: !!component.value?.parentId,
    });
  } else {
    const { position, size } =
      componentsStore.initMinimzedElementPositionAndSize(props.group.uniqueKey);
    componentsStore.updateMinimzedElementPositionAndSize({
      uniqueKey: props.group.uniqueKey,
      position,
      size,
    });
    trackEvent("collapse-components", {
      source: "diagram-group",
      schemaVariantName: component.value?.schemaVariantName,
      schemaName: component.value?.schemaName,
      hasParent: !!component.value?.parentId,
    });
  }
};

// RENAME ON DIAGRAM STUFF
const renameHitboxSelfRect = ref();

onUpdated(() => {
  renameHitboxSelfRect.value = titleTextRef.value?.getNode()?.getSelfRect();
});

const renameHitbox = computed(() => {
  if (titleTextRef.value) {
    const raw =
      renameHitboxSelfRect.value ||
      titleTextRef.value?.getNode()?.getSelfRect();

    if (raw) {
      const textWidth = titleTextRef.value?.getNode()?.getTextWidth();

      const width = textWidth ? textWidth + 4 : raw.width;

      const box = {
        ...raw,
        width,
        height: GROUP_TITLE_FONT_SIZE + 6,
        x: 32 + GROUP_HEADER_ICON_SIZE,
        y: 4,
      };
      return box;
    }
  }

  return {
    x: 32 + GROUP_HEADER_ICON_SIZE,
    y: 4,
    width: headerWidth.value - 14 - GROUP_HEADER_ICON_SIZE * 2,
    height: GROUP_TITLE_FONT_SIZE + 6,
  };
});

const renaming = ref(false);

function mouseOverRename(evt: KonvaEventObject<MouseEvent>) {
  if (props.isSelected) {
    onMouseOver(evt, "rename");
  }
}

function mouseOutRename(evt: KonvaEventObject<MouseEvent>) {
  if (props.isSelected) {
    onMouseOver(evt);
  }
}

function renameIfSelected(e: KonvaEventObject<MouseEvent>) {
  if (
    props.isSelected &&
    e.evt.button === 0 &&
    componentsStore.hoveredComponentMeta?.type === "rename"
  ) {
    rename();
  }
}

function rename() {
  componentsStore.setHoveredComponentId(componentId.value, {
    type: "rename",
  });
  renaming.value = true;
  emit("rename", () => {
    renaming.value = false;
  });
}
</script>
