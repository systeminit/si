<template>
  <!-- FIXME(nick,victor): find a way to remove z levels from here. This is needed for interaction with the canvas to work. -->
  <div
    class="z-20 flex flex-col dark:text-white border-[#DBDBDB] dark:border-[#525252] bg-white dark:bg-[#333333] w-40 lg:w-56 xl:w-72 pointer-events-auto relative transition-all"
    :class="panelClasses"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { toRef, computed } from "vue";

const props = defineProps<{
  side: "right" | "left";
  hidden?: boolean;
}>();
const side = toRef(props, "side");

const panelClasses = computed(() => {
  const classes: Record<string, boolean> = {};
  if (side.value == "left") {
    classes["border-r-2"] = true;
    classes[props.hidden ? "right-72" : "right-0"] = true;
  } else {
    classes["border-l-2"] = true;
    classes[props.hidden ? "left-72" : "left-0"] = true;
  }

  return classes;
});
</script>
