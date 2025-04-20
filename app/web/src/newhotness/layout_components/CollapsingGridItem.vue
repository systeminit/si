<template>
  <div class="scrollable">
    <h3
      class="sticky top-0 cursor-pointer text-lg font-bold"
      @click="openState.toggle"
    >
      <slot name="header" />
    </h3>
    <slot />
  </div>
</template>

<script lang="ts" setup>
import { useToggle } from "../logic_composables/toggle_containers";

const openState = useToggle();

defineExpose({
  openState,
});
</script>
