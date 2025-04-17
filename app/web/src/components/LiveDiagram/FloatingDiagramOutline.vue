<template>
  <FloatingPanel
    title="Diagram Outline"
    :isOpen="isOpen"
    :width="350"
    :height="600"
    :position="position || 'top-left'"
    @close="emit('close')"
  >
    <DiagramOutline
      :actionsAreRunning="false"
      :leftDrawerOpen="false"
      :toggleDrawer="() => {}"
      @right-click-item="onOutlineRightClick"
    />
  </FloatingPanel>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import FloatingPanel from "./FloatingPanel.vue";
import DiagramOutline from "../DiagramOutline/DiagramOutline.vue";
import { RightClickElementEvent } from "../ModelingDiagram/diagram_types";

const props = defineProps<{
  isOpen: boolean;
  position?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "right-click-item", event: RightClickElementEvent): void;
}>();

function onOutlineRightClick(event: RightClickElementEvent) {
  emit("right-click-item", event);
}
</script>
