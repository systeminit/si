<template>
  <!-- FIXME(nick,victor): find a way to remove z levels from here. This is needed for interaction with the canvas to work. -->
  <div :class="panelClasses">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { toRef, computed } from "vue";

const props = withDefaults(
  defineProps<{
    side: "right" | "left";
    hidden?: boolean;
    classes?: string;
  }>(),
  {
    classes:
      "z-20 flex flex-col dark:text-white border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 w-72 xl:w-96 pointer-events-auto relative transition-all",
  },
);
const side = toRef(props, "side");

const panelClasses = computed(() => {
  const classes: Record<string, boolean> = {};
  if (side.value == "left") {
    classes["border-r-2"] = true;
    classes[props.hidden ? "right-96" : "right-0"] = true;
  } else {
    classes["border-l-2"] = true;
    classes[props.hidden ? "left-96" : "left-0"] = true;
  }
  const propClasses = props.classes.split(" ");

  propClasses.forEach((c) => {
    classes[c] = true;
  });

  return classes;
});
</script>
