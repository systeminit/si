<template>
  <div id="app-layout" :class="classes">
    <slot />
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";

const props = defineProps({
  pageMode: {
    type: String as PropType<"scroll" | "fullscreen">,
    default: "fullscreen",
  },
});

const classes = computed(() => {
  if (props.pageMode === "fullscreen") {
    return "overflow-hidden flex flex-col w-full h-screen select-none";
  }
  return "flex flex-col items-center justify-center min-h-screen w-full bg-shade-100";
});
</script>
