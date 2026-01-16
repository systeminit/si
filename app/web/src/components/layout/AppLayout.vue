<template>
  <!-- this div/id is used as the attaching point for FloatingVue (tooltips) -->
  <div
    id="app-layout"
    :class="
      clsx(
        'flex flex-col w-full dark:bg-black dark:text-white',
        pageMode === 'fullscreen' && 'overflow-hidden h-screen',
        pageMode === 'scroll' && 'items-center justify-center min-h-screen',
        pageMode === 'modal' && 'items-center justify-center min-h-screen p-4 sm:p-12 flex flex-col',
      )
    "
  >
    <slot />
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed, PropType } from "vue";
import { useHead } from "@vueuse/head";

const props = defineProps({
  pageMode: {
    type: String as PropType<"scroll" | "fullscreen" | "modal">,
    default: "fullscreen",
  },
});

useHead(
  computed(() => ({
    bodyAttrs: {
      class: props.pageMode === "fullscreen" ? "overflow-hidden" : "",
    },
  })),
);
</script>
