<template>
  <Teleport to="body">
    <Transition
      name="wipe"
      @after-enter="onWipeOpenDone"
      @after-leave="onWipeCloseDone"
    >
      <div
        v-if="state === 'running' || state === 'done'"
        :class="
          clsx(
            'wipe',
            'fixed z-80 top-0 left-0 w-screen h-screen overflow-hidden flex flex-col items-center justify-center bg-neutral-50 dark:bg-neutral-900',
          )
        "
      >
        <slot name="afterWipe" :state="state" />
      </div>
    </Transition>
    <div
      :class="
        clsx(
          'fixed z-100',
          state !== 'running' && 'hidden',
          state === 'running' && 'translate-x-[-50%] translate-y-[-50%]',
        )
      "
      :style="{
        top: `${origin.y}px`,
        left: `${origin.x}px`,
      }"
    >
      <slot name="duringWipe" />
    </div>
  </Teleport>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import clsx from "clsx";
import { useHead } from "@vueuse/head";
import { createDeferredPromise, DeferredPromise } from "@si/ts-lib";

export type WipeState = "idle" | "running" | "done" | "exiting";
type Position = { x: number; y: number };

const state = ref("idle" as WipeState);
const origin = ref<Position>({ x: 0, y: 0 });
const originX = computed(() => `${Math.round(origin.value.x)}px`);
const originY = computed(() => `${Math.round(origin.value.y)}px`);
// Unfortunately, v-bind, Teleport, and Transition do not all cooperate together!
// So we're just injecting global variables to hold the wipe origin for the CSS below.
// Ask Theo or Wendy or Fletcher for more details!
useHead(
  computed(() => ({
    style: [
      {
        children: `:root {--si-wipe-origin-x: ${originX.value}; --si-wipe-origin-y: ${originY.value};}`,
      },
    ],
  })),
);
let openDonePromise: DeferredPromise;
let closeDonePromise: DeferredPromise;

const onWipeOpenDone = () => {
  state.value = "done";
  openDonePromise.resolve();
};

const onWipeCloseDone = () => {
  state.value = "idle";
  closeDonePromise.resolve();
};

const open = async (openAt: HTMLElement | Position) => {
  if (openAt instanceof HTMLElement) {
    // origin based on a given HTMLElement
    const openAtRect = openAt.getBoundingClientRect();

    origin.value = {
      x: openAtRect.left + openAtRect.width / 2,
      y: openAtRect.top + openAtRect.height / 2,
    };
  } else {
    // origin based on a given x, y position
    origin.value = openAt;
  }

  // run the wipe!
  // Wendy - there's a setTimeout here to prevent a bug where the origin doesn't set correctly
  // If you can figure out a better solution to this issue, feel free to change this!
  setTimeout(() => {
    state.value = "running";
  }, 20);
  openDonePromise = createDeferredPromise();
  return openDonePromise.promise;
};

const close = () => {
  origin.value = {
    x: window.innerWidth / 2,
    y: window.innerHeight / 2,
  };

  state.value = "exiting";
  closeDonePromise = createDeferredPromise();
  return closeDonePromise.promise;
};

defineExpose({ open, close, state });
</script>

<style scoped>
.wipe-enter-from {
  clip-path: circle(0px at var(--si-wipe-origin-x) var(--si-wipe-origin-y));
}
.wipe-enter-to {
  clip-path: circle(250vw at var(--si-wipe-origin-x) var(--si-wipe-origin-y));
}
.wipe-enter-active {
  transition: clip-path 1.312s;
}
.wipe-leave-from {
  clip-path: circle(250vw at 50vw 50vh);
}
.wipe-leave-to {
  clip-path: circle(0px at 50vw 50vh);
}
.wipe-leave-active {
  transition: clip-path 1.312s;
}
</style>
