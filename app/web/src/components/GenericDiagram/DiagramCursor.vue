<template>
  <v-group
    :config="{
      id: `cursor-${cursor.userId}`,
      x: props.cursor.x,
      y: props.cursor.y,
    }"
  >
    <!-- package/type icon -->
    <DiagramIcon
      icon="cursor-array-rays"
      :color="cursor.color || colorPrefix"
      :size="40"
      :x="0"
      :y="0"
      origin="center"
    />
    <v-text
      :config="{
        x: 10,
        y: 10,
        text: cursor.name,
        fill: cursor.color || colorPrefix,
        fontStyle: 'bold',
      }"
    >
    </v-text>
  </v-group>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { useTheme } from "@si/vue-lib/design-system";
import { DiagramCursorDef } from "@/store/presence.store";
import DiagramIcon from "./DiagramIcon.vue";

const props = defineProps<{
  cursor: DiagramCursorDef;
}>();

const { theme } = useTheme();
const colorPrefix = computed(() => {
  if (theme.value === "dark") return "white";
  return "black";
});
</script>
