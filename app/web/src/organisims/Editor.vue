<template>
  <div class="flex flex-col w-full h-full editor">
    <PanelTree />
  </div>
</template>

<script setup lang="ts">
import PanelTree from "@/organisims/PanelTree.vue";
</script>

<style scoped>
.editor {
  border-top: 1px solid #242424;
}
</style>
