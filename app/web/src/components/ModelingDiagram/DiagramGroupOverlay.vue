<template>
  <v-group
    ref="groupRef"
    :config="{
      id: group.uniqueKey,
      x: position.x,
      y: position.y,
      listening: false,
    }"
  >
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

    <DiagramIcon
      v-if="isDeleted"
      icon="minus-square"
      :color="getToneColorHex('destructive')"
      :size="deletedIconSize"
      :x="0"
      :y="nodeHeight / 2"
    />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, nextTick, PropType, ref, watch } from "vue";
import * as _ from "lodash-es";
import { Tween } from "konva/lib/Tween";
import { Vector2d } from "konva/lib/types";
import { getToneColorHex } from "@si/vue-lib/design-system";
import {
  CORNER_RADIUS,
  GROUP_HEADER_BOTTOM_MARGIN,
} from "@/components/ModelingDiagram/diagram_constants";
import { DiagramGroupData, Size2D } from "./diagram_types";
import DiagramIcon from "./DiagramIcon.vue";

const props = defineProps({
  group: {
    type: Object as PropType<DiagramGroupData>,
    required: true,
  },
  tempPosition: {
    type: Object as PropType<Vector2d>,
  },
  tempSize: {
    type: Object as PropType<Size2D>,
  },
  isHovered: Boolean,
  isSelected: Boolean,
});

const titleTextRef = ref();
const groupRef = ref();

const size = computed(
  () => props.tempSize || props.group.def.size || { width: 500, height: 500 },
);

const nodeWidth = computed(() => size.value.width);
const halfWidth = computed(() => nodeWidth.value / 2);

const overlayIconSize = computed(() => nodeWidth.value / 3);

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
}

const nodeHeaderHeight = computed(() => headerTextHeight.value);
const nodeBodyHeight = computed(() => size.value.height);
const nodeHeight = computed(
  () =>
    nodeHeaderHeight.value + GROUP_HEADER_BOTTOM_MARGIN + nodeBodyHeight.value,
);

const position = computed(() => props.tempPosition || props.group.def.position);
const isDeleted = computed(() => props.group?.def.changeStatus === "deleted");
const deletedIconSize = computed(() =>
  Math.min(nodeHeight.value, nodeWidth.value, 300),
);

const overlay = ref();
watch([() => props.group.def.isLoading, overlay], ([isLoading]) => {
  if (_.isNil(overlay)) return;
  const node = overlay.value.getNode();

  const transition = new Tween({
    node,
    duration: 0.1,
    opacity: isLoading ? 1 : 0,
  });

  transition.play();
});
</script>
