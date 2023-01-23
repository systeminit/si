<template>
  <div
    :class="tabClasses"
    class="flex w-80 border-black border-l h-full place-content-center pl-2 pr-2"
  >
    <div class="flex items-center">
      <slot name="icon" />
    </div>
    <div class="flex text-sm place-content-center items-center pl-2">
      <slot name="name" />
    </div>
    <div class="flex grow justify-end place-content-center items-center gap-2">
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
