<template>
  <div
    class="flex w-72 border-black border-l h-full place-content-center pl-2 pr-2"
    :class="tabClasses"
  >
    <div class="flex w-6">
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

<script setup lang="ts">
import { toRef, computed } from "vue";

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
