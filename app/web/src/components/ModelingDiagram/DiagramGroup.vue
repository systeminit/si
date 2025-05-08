<template>
  <v-group v-if="!occlude">
    <v-rect
      v-if="debug"
      :config="{
        width: renderRect.width,
        height: renderRect.height,
        x: renderRect.x,
        y: renderRect.y,
        fill: 'red',
        opacity: 0.2,
      }"
    />
    <v-group
      ref="groupRef"
      :config="{
        id: group.uniqueKey,
        x: irect.x,
        y: irect.y,
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
          y: -4 - NODE_HEADER_HEIGHT - GROUP_HEADER_BOTTOM_MARGIN,
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
      <!--  left side handle  -->
      <v-line
        :config="{
          points: [
            -nodeWidth / 2,
            -(NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN),
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
            -(NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN),
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
            -(NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN),
            nodeWidth / 2,
            -(NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN),
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
          y: -(NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN),
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
          y: -(NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN),
        }"
        @mouseover="onResizeHover('top-right', $event)"
        @mouseout="onMouseOut"
      />

      <!-- sockets -->
      <v-group v-if="hideDetails === 'show'">
        <v-group
          ref="socketsLeftRef"
          :config="{
            x: leftSockets.x,
            y: leftSockets.y,
          }"
        >
          <DiagramNodeSocket
            v-for="socket in leftSockets.sockets"
            :key="socket.uniqueKey"
            :socket="socket"
            :position="socket.position"
            :connectedEdges="
              connectedEdgesBySocketKey[
                featureFlagsStore.SIMPLE_SOCKET_UI && !socket.def.isManagement
                  ? `${group.def.id}-inputsocket`
                  : socket.uniqueKey
              ]
            "
            :nodeWidth="nodeWidth"
            :isDeleted="isDeleted"
            @hover:start="onSocketHoverStart(socket)"
            @hover:end="onSocketHoverEnd(socket)"
          />
        </v-group>
        <v-group
          ref="socketsRightRef"
          :config="{
            x: rightSockets.x,
            y: rightSockets.y,
          }"
        >
          <DiagramNodeSocket
            v-for="socket in rightSockets.sockets"
            :key="socket.uniqueKey"
            :socket="socket"
            :position="socket.position"
            :connectedEdges="
              connectedEdgesBySocketKey[
                featureFlagsStore.SIMPLE_SOCKET_UI && !socket.def.isManagement
                  ? `${group.def.id}-outputsocket`
                  : socket.uniqueKey
              ]
            "
            :nodeWidth="nodeWidth"
            :isDeleted="isDeleted"
            @hover:start="onSocketHoverStart(socket)"
            @hover:end="onSocketHoverEnd(socket)"
          />
        </v-group>
      </v-group>

      <!-- header -->
      <v-group
        :config="{
          x: -halfWidth,
          y: -NODE_HEADER_HEIGHT - GROUP_HEADER_BOTTOM_MARGIN,
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
            height: NODE_HEADER_HEIGHT,
          }"
        />

        <DiagramIcon
          v-if="hideDetails !== 'hide'"
          :icon="COMPONENT_TYPE_ICONS[group.def.componentType]"
          origin="top-left"
          :size="32"
          :x="2"
          :y="5"
          :color="colors.headerText"
        />

        <!-- header text -->

        <!-- rename hitbox -->
        <v-rect
          v-if="hideDetails !== 'hide'"
          :config="{
            ...renameHitbox,
            ...(debug && { fill: 'red' }),
            cornerRadius: 2,
            strokeWidth: 2,
            stroke: renameHovered ? SELECTION_COLOR : 'transparent',
          }"
          @mouseout="mouseOutRename"
          @mousemove="mouseOverRename"
          @mouseover="mouseOverRename"
          @click="renameIfSelected"
          @dblclick="rename"
        />

        <!-- component name -->
        <v-text
          v-if="hideDetails !== 'hide'"
          ref="titleTextRef"
          :config="{
            x: GROUP_HEADER_ICON_SIZE - 2,
            y: 2,
            verticalAlign: 'top',
            align: 'left',
            width:
              hideDetails === 'show'
                ? headerWidth - GROUP_HEADER_ICON_SIZE * 2
                : headerWidth - GROUP_HEADER_ICON_SIZE,
            text: group.def.title,
            padding: 6,
            fill: colors.headerText,
            fontSize:
              hideDetails === 'show'
                ? GROUP_TITLE_FONT_SIZE
                : GROUP_TITLE_FONT_SIZE_TINY,
            fontStyle: renameHovered ? 'italic bold' : 'bold',
            fontFamily: DIAGRAM_FONT_FAMILY,
            listening: false,
            wrap: 'none',
            ellipsis: true,
          }"
        />

        <!-- component type and child count -->
        <v-text
          v-if="hideDetails === 'show'"
          ref="subtitleTextRef"
          :config="{
            x: GROUP_HEADER_ICON_SIZE - 2,
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
        v-if="parentComponentId && hideDetails === 'show'"
        icon="frame"
        :size="16"
        :x="-halfWidth + 12"
        :y="nodeBodyHeight - 12"
        :color="colors.parentColor"
        @mouseover="(e) => onMouseOver(e, 'parent')"
        @mouseout="onMouseOut"
      />

      <!-- status icons -->
      <v-group
        v-if="statusIcons?.length && hideDetails === 'show'"
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
            (statusIcon.color || statusIcon.tone) && !isDeleted
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

      <!-- upgrade icon -->
      <DiagramIcon
        v-if="group.def.canBeUpgraded && hideDetails === 'show'"
        :color="getToneColorHex('action')"
        :size="24 + (diffIconHover ? 4 : 0)"
        :x="halfWidth - GROUP_HEADER_ICON_SIZE - 36 / 2"
        :y="
          -NODE_HEADER_HEIGHT +
          GROUP_HEADER_ICON_SIZE / 2 -
          GROUP_HEADER_BOTTOM_MARGIN +
          (NODE_HEADER_HEIGHT - GROUP_HEADER_ICON_SIZE) / 2
        "
        icon="bolt"
        origin="center"
      />

      <!-- added/modified indicator -->
      <DiagramIcon
        v-if="(isAdded || isModified || isDeleted) && hideDetails === 'show'"
        :icon="topRightIcon"
        :color="topRightIconColor"
        shadeBg
        :size="GROUP_HEADER_ICON_SIZE + (diffIconHover ? 8 : 0)"
        :x="halfWidth - GROUP_HEADER_ICON_SIZE / 2"
        :y="
          -NODE_HEADER_HEIGHT +
          GROUP_HEADER_ICON_SIZE / 2 -
          GROUP_HEADER_BOTTOM_MARGIN +
          (NODE_HEADER_HEIGHT - GROUP_HEADER_ICON_SIZE) / 2
        "
        origin="center"
        @click="onClick('diff')"
        @mouseover="diffIconHover = true"
        @mouseout="diffIconHover = false"
      />

      <!-- debug text to show the group width and height -->
      <v-text
        v-if="debug"
        :config="{
          x: -nodeWidth / 2,
          y: -(NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN + 22),
          verticalAlign: 'top',
          align: 'left',
          text: `x: ${irect.x} y: ${irect.y} width: ${irect.width} height: ${irect.height}`,
          fill: 'red',
          fontStyle: 'bold',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
        }"
      />
    </v-group>
  </v-group>
</template>

<script lang="ts" setup>
import {
  computed,
  onUpdated,
  PropType,
  ref,
  watch,
  onMounted,
  nextTick,
} from "vue";
import * as _ from "lodash-es";
import tinycolor from "tinycolor2";
import { IRect } from "konva/lib/types";
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
  NODE_HEADER_HEIGHT,
  GROUP_HEADER_ICON_SIZE,
  GROUP_RESIZE_HANDLE_SIZE,
  GROUP_TITLE_FONT_SIZE,
  GROUP_TITLE_FONT_SIZE_TINY,
  SELECTION_COLOR,
  NODE_SUBTITLE_TEXT_HEIGHT,
  SOCKET_MARGIN_TOP,
  NODE_PADDING_BOTTOM,
  DetailsMode,
} from "@/components/ModelingDiagram/diagram_constants";
import {
  QualificationStatus,
  statusIconsForComponent,
} from "@/store/qualifications.store";
import { useViewsStore } from "@/store/views.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  DiagramDrawEdgeState,
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramGroupData,
  DiagramSocketData,
  DiagramSocketEdgeData,
  ElementHoverMeta,
  SideAndCornerIdentifiers,
} from "./diagram_types";
import { useDiagramContext } from "./ModelingDiagram.vue";
import DiagramIcon from "./DiagramIcon.vue";
import { checkRectanglesOverlap } from "./utils/math";

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
  debug: Boolean,
  hideDetails: { type: String as PropType<DetailsMode> },
  occlusionRect: { type: Object as PropType<IRect> },
});

const featureFlagsStore = useFeatureFlagsStore();
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
  (e: "rename", v: () => void): void;
}>();

const { theme } = useTheme();

const titleTextRef = ref();
const subtitleTextRef = ref();
const groupRef = ref();
const socketsLeftRef = ref();
const socketsRightRef = ref();

const componentsStore = useComponentsStore();
const viewStore = useViewsStore();

const irect = computed(() => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const r = viewStore.groups[props.group.def.id]!;

  const minimum =
    NODE_SUBTITLE_TEXT_HEIGHT +
    SOCKET_MARGIN_TOP +
    NODE_PADDING_BOTTOM +
    30 +
    NODE_HEADER_HEIGHT;

  return {
    x: r.x,
    y: r.y,
    width: Math.max(r.width, minimum),
    height: Math.max(r.height, minimum),
  };
});

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
    (child) => componentsStore.allComponentsById[child],
  );

  const undeletedChildren = _.filter(mappedChildren, (child) =>
    _.isNil(child?.def.deletedInfo),
  );

  return undeletedChildren.length;
});

const nodeWidth = computed(() => irect.value.width);
const halfWidth = computed(() => nodeWidth.value / 2);
const headerWidth = computed(() =>
  !props.group.def.changeStatus ||
  props.group.def.changeStatus === "unmodified" ||
  props.hideDetails !== "show"
    ? nodeWidth.value
    : nodeWidth.value - GROUP_HEADER_ICON_SIZE - 4,
);

const leftSockets = computed(() =>
  props.group.layoutLeftSockets(nodeWidth.value),
);
const rightSockets = computed(() =>
  props.group.layoutRightSockets(nodeWidth.value),
);

const connectedEdgesBySocketKey = computed(() => {
  const lookup: Record<DiagramElementUniqueKey, DiagramEdgeData[]> = {};
  _.each(props.connectedEdges, (edge) => {
    if (featureFlagsStore.SIMPLE_SOCKET_UI && !edge.def.isManagement) {
      if (edge.toNodeKey === props.group.uniqueKey) {
        lookup[`${props.group.def.id}-inputsocket`] ||= [];
        lookup[`${props.group.def.id}-inputsocket`]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
      }
      if (edge.fromNodeKey === props.group.uniqueKey) {
        lookup[`${props.group.def.id}-outputsocket`] ||= [];
        lookup[`${props.group.def.id}-outputsocket`]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
      }
      // If it's not the simple socket UI, subscriptions are not connected
    } else if (edge instanceof DiagramSocketEdgeData) {
      lookup[edge.fromSocketKey] ||= [];
      lookup[edge.fromSocketKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
      lookup[edge.toSocketKey] ||= [];
      lookup[edge.toSocketKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
    }
  });
  return lookup;
});

const nodeBodyHeight = computed(() => irect.value.height);
const nodeHeight = computed(
  () => NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN + nodeBodyHeight.value,
);

const colors = computed(() => {
  const primaryColor = tinycolor(props.group.def.color || DEFAULT_NODE_COLOR);
  const bodyBgHsl = primaryColor.toHsl();
  const borderHsl = primaryColor.toHsl();

  let bodyText = theme.value === "dark" ? "#FFF" : "#000";

  const parentColorHsl = tinycolor(
    componentsStore.allComponentsById[parentComponentId.value || ""]?.def
      .color || "#FFF",
  ).toHsl();

  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  if (isDeleted.value) {
    bodyBgHsl.s = 0.1;
    borderHsl.s = 0.5;
    bodyText = "#999";
    parentColorHsl.s = 0.5;
  }

  const bodyBg = tinycolor(bodyBgHsl).toRgbString();
  const border = tinycolor(borderHsl).toRgbString();
  const parentColor = tinycolor(parentColorHsl).toRgbString();

  let headerText;
  if (primaryColor.toHsl().l < 0.5) {
    headerText = "#FFF";
  } else {
    headerText = "#000";
  }

  return {
    headerBg: border,
    icon: "#000",
    headerText,
    bodyBg,
    labelText: bodyText,
    bodyText,
    parentColor,
  };
});

function onMouseOver(evt: KonvaEventObject<MouseEvent>, type?: string) {
  evt.cancelBubble = true;
  viewStore.setHoveredComponentId(
    componentId.value,
    type ? ({ type } as ElementHoverMeta) : undefined,
  );
}

function onResizeHover(
  direction: SideAndCornerIdentifiers,
  evt: KonvaEventObject<MouseEvent>,
) {
  evt.cancelBubble = true;
  viewStore.setHoveredComponentId(componentId.value, {
    type: "resize",
    direction,
  });
}

function onSocketHoverStart(socket: DiagramSocketData) {
  viewStore.setHoveredComponentId(componentId.value, {
    type: "socket",
    socket,
  });
}

function onSocketHoverEnd(_socket: DiagramSocketData) {
  viewStore.setHoveredComponentId(null);
}

function onMouseOut() {
  viewStore.setHoveredComponentId(null);
}

function onClick(detailsTabSlug: string) {
  viewStore.setSelectedComponentId(componentId.value, {
    detailsTab: detailsTabSlug,
  });
}

const highlightParent = computed(() => {
  if (!viewStore.hoveredComponent) return false;
  if (viewStore.hoveredComponentMeta?.type !== "parent") return false;
  return (
    viewStore.hoveredComponent.def.ancestorIds?.includes(componentId.value) ||
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
        x: GROUP_HEADER_ICON_SIZE + 2,
        y: 4,
      };
      return box;
    }

    // we only reach this point if the rename input field is active
    return {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
    };
  }

  return {
    x: 32 + GROUP_HEADER_ICON_SIZE,
    y: 4,
    width: headerWidth.value - 14 - GROUP_HEADER_ICON_SIZE * 2,
    height: GROUP_TITLE_FONT_SIZE + 6,
  };
});

const renaming = ref(false);
const renameHoverState = ref(false);
const fixCursorToText = ref(false);

const renameHovered = computed(
  () =>
    (viewStore.hoveredComponentMeta?.type === "rename" &&
      viewStore.hoveredComponentId === props.group.def.id) ||
    renameHoverState.value,
);

const selectedAndRenameHovered = computed(
  () =>
    props.isSelected &&
    viewStore.hoveredComponentMeta?.type === "rename" &&
    viewStore.hoveredComponentId === props.group.def.id &&
    renameHoverState.value,
);

watch(
  () => props.isSelected,
  (isSelected) => {
    if (isSelected && renameHovered.value) {
      fixCursorToText.value = true;
    }
  },
);

function mouseOverRename(evt: KonvaEventObject<MouseEvent>) {
  if (props.isSelected) {
    onMouseOver(evt, "rename");
  }
  renameHoverState.value = true;
}

function mouseOutRename() {
  if (props.isSelected) {
    onMouseOut();
  }
  renameHoverState.value = false;
  fixCursorToText.value = false;
}

function renameIfSelected(e: KonvaEventObject<MouseEvent>) {
  if (e.evt.button === 0 && selectedAndRenameHovered.value) {
    rename();
  } else if (fixCursorToText.value) {
    fixCursorToText.value = false;
    viewStore.setHoveredComponentId(componentId.value, {
      type: "rename",
    } as ElementHoverMeta);
  }
}

function rename() {
  viewStore.setHoveredComponentId(componentId.value, {
    type: "rename",
  });
  renaming.value = true;
  fixCursorToText.value = false;
  emit("rename", () => {
    renaming.value = false;
  });
}

const RENDER_RECT_EDGE = 8;

const renderRect = computed(() => ({
  x: irect.value.x - (irect.value.width / 2 + RENDER_RECT_EDGE),
  y:
    irect.value.y -
    (NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN + RENDER_RECT_EDGE),
  width: irect.value.width + RENDER_RECT_EDGE * 2,
  height:
    irect.value.height +
    (NODE_HEADER_HEIGHT + GROUP_HEADER_BOTTOM_MARGIN + RENDER_RECT_EDGE * 2),
}));

const occlude = computed(() => {
  if (!props.occlusionRect) {
    return false;
  } else {
    const o = !checkRectanglesOverlap(renderRect.value, props.occlusionRect);
    return o;
  }
});

const cache = () => {
  if (
    !featureFlagsStore.DIAGRAM_OPTIMIZATION_2 ||
    occlude.value ||
    props.hideDetails !== "show"
  )
    return;

  // this allows us to fire the cache via the watcher on occlusion
  nextTick(() => {
    const left = socketsLeftRef.value?.getNode();
    const right = socketsRightRef.value?.getNode();
    if (left && right) {
      left.cache({
        pixelRatio: diagramContext.zoomLevel.value,
      });
      right.cache({
        pixelRatio: diagramContext.zoomLevel.value,
      });
    }
  });
};

const clearCache = () => {
  if (!featureFlagsStore.DIAGRAM_OPTIMIZATION_2) return;

  const left = socketsLeftRef.value?.getNode();
  const right = socketsRightRef.value?.getNode();
  if (left && right) {
    left.clearCache();
    right.clearCache();
  }
};

onMounted(() => {
  cache();
});

// cache nodes when they come on screen
watch(
  () => occlude.value,
  () => {
    cache();
  },
);

// re-run caching when switching between semantic zoom states
watch(
  () => props.hideDetails,
  () => {
    cache();
  },
);

// re-run caching when sockets change
watch(
  () => connectedEdgesBySocketKey.value,
  () => {
    cache();
  },
  { deep: true },
);

// re-run caching when an edge is being drawn
watch(
  () => diagramContext.drawEdgeState.value,
  () => {
    cache();
  },
  { deep: true },
);

// re-run caching when zoomed in close enough
watch(
  () => diagramContext.zoomLevel.value,
  (zoomLevel) => {
    if (zoomLevel > 1) {
      cache();
    }
  },
  { deep: true },
);

// re-run caching when switching color themes
watch(
  () => theme.value,
  () => {
    cache();
  },
);

defineExpose({ clearCache });
</script>
