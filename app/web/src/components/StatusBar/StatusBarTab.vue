<template>
  <div
    :class="tabClasses"
    class="flex p-xs"
    @click="(e) => props.click && props.click()"
  >
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

type clickFn = () => void;
const props = defineProps<{ selected: boolean; click?: clickFn }>();
const selected = toRef(props, "selected");

const tabClasses = computed(() => {
  const result: Record<string, boolean> = {};
  if (selected.value) {
    result["bg-black"] = true;
  }
  return result;
});
</script>
