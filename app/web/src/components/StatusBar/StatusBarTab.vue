<template>
  <div :class="tabClasses" class="flex p-xs">
    <div class="flex items-center">
      <slot name="icon" />
    </div>
    <div class="flex text-sm items-center pl-xs mr-lg">
      <slot name="name" />
    </div>
    <div class="flex grow justify-end items-center gap-1">
      <slot name="summary" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, toRef } from "vue";

const props = defineProps<{ selected: boolean }>();
const selected = toRef(props, "selected");

const tabClasses = computed(() => {
  const result: Record<string, boolean> = {};
  if (selected.value) {
    result["bg-black"] = true;
  }
  return result;
});
</script>
