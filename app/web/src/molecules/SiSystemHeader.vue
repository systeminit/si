<template>
  <div class="flex">
    <div class="mt-1 font-medium align-middle">
      <slot name="title"></slot>
    </div>
    <div class="mt-1 mr-4 align-middle">
      <SystemWidget />
    </div>
  </div>
</template>

<script setup lang="ts">
import SystemWidget from "@/organisims/SystemWidget.vue";
</script>

<style scoped>
.header-background {
  background-color: #171717;
}
</style>
