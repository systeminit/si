<template>
  <div class="flex">
    <div class="font-medium align-middle">
      <slot name="title"></slot>
    </div>
    <div class="mr-4 align-middle">
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
