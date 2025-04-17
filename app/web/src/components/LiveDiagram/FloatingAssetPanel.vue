<template>
  <FloatingPanel
    title="Assets"
    :isOpen="isOpen"
    :width="350"
    :height="600"
    :position="position || 'top-left'"
    @close="emit('close')"
  >
    <AssetPalette />
  </FloatingPanel>
</template>

<script lang="ts" setup>
import FloatingPanel from "./FloatingPanel.vue";
import AssetPalette from "../AssetPalette.vue";

const props = defineProps<{
  isOpen: boolean;
  position?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>