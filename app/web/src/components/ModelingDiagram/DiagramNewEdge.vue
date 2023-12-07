<template>
  <v-line
    v-if="points"
    :config="{
      points,
      stroke: strokeColor,
      strokeWidth: 3,
      listening: false, // no need to listen for mouse events
    }"
  />
</template>

<script lang="ts" setup>
import { Vector2d } from "konva/lib/types";
import { computed, PropType } from "vue";
import { useTheme } from "@si/vue-lib/design-system";

const props = defineProps({
  fromPoint: {
    type: Object as PropType<Vector2d>,
    default: undefined,
  },
  toPoint: {
    type: Object as PropType<Vector2d>,
  },
});

const { theme } = useTheme();

const points = computed(() => {
  if (!props.fromPoint || !props.toPoint) return;
  return [
    props.fromPoint.x,
    props.fromPoint.y,
    props.toPoint.x,
    props.toPoint.y,
  ];
});

const strokeColor = computed(() => (theme.value === "dark" ? "#FFF" : "#000"));
</script>
