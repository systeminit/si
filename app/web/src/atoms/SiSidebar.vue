<template>
  <div
    class="flex flex-col dark:text-white border-[#DBDBDB] dark:border-[#525252] bg-white dark:bg-[#333333] w-40 lg:w-56 xl:w-72 pointer-events-auto"
    :class="panelClasses"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { toRef, computed } from "vue";

const props = defineProps<{ side: "right" | "left" }>();
const side = toRef(props, "side");

const panelClasses = computed(() => {
  const classes: Record<string, boolean> = {};
  if (side.value == "left") {
    classes["border-r-2"] = true;
  } else {
    classes["border-l-2"] = true;
  }
  return classes;
});
</script>
