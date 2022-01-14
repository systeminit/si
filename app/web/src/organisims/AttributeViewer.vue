<template>
  <div class="w-full h-full">
    <Viewer :component-id="props.componentId" />
  </div>
</template>

<script setup lang="ts">
import Viewer from "./AttributeViewer/Viewer.vue";

const props = defineProps<{
  componentId: number;
}>();
</script>
