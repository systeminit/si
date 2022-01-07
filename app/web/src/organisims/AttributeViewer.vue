<template>
  <div class="w-full h-full">
    <Viewer :componentId="props.componentId"/>
  </div>
</template>

<script setup lang="ts">

import Viewer from "./AttributeViewer/Viewer.vue";

const props = defineProps<{
  componentId: number
}>();

</script>