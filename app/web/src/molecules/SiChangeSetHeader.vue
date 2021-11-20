<template>
  <div
    class="flex flex-row items-center justify-between flex-grow-0 flex-shrink-0 h-12 header-background"
  >
    <div class="mt-1 ml-8 font-medium align-middle">
      <slot name="title"></slot>
    </div>
    <div class="mt-1 mr-8 align-middle">
      <ChangeSetWidget />
    </div>
  </div>
</template>

<script setup lang="ts">
import ChangeSetWidget from "@/organisims/ChangeSetWidget.vue";
</script>

<style scoped>
.header-background {
  background-color: #171717;
}
</style>
