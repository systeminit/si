<template>
  <div
    ref="scrollDivRef"
    :class="
      clsx(
        'overflow-auto overflow-x-auto scrollable scroll-slot',
        hideScrollbar && 'scrollbar-hidden',
      )
    "
  >
    <slot />
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { onBeforeUnmount, onMounted, ref } from "vue";

defineProps({
  hideScrollbar: { type: Boolean },
});

const horizontalScroll = (evt: WheelEvent) => {
  evt.preventDefault();

  const div = scrollDivRef.value;
  if (!div) return;

  // we do both so that it scrolls regardless of if you hold shift or not
  div.scrollLeft += evt.deltaY;
  div.scrollLeft += evt.deltaX;
};

const scrollDivRef = ref<HTMLElement | null>(null);
defineExpose({ scrollElement: scrollDivRef });

onMounted(() => {
  if (scrollDivRef.value) {
    scrollDivRef.value.addEventListener("wheel", horizontalScroll, {
      passive: false,
    });
  }
});

onBeforeUnmount(() => {
  if (scrollDivRef.value) {
    scrollDivRef.value.removeEventListener("wheel", horizontalScroll);
  }
});
</script>
