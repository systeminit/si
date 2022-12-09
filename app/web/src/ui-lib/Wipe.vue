<template>
  <Teleport to="body">
    <Transition
      enter-from-class="w-0 h-0"
      enter-to-class="w-[250vw] h-[250vw]"
      enter-active-class="duration-[20000ms] ease-in rounded-full"
      leave-from-class="w-[250vw] h-[250vw]"
      leave-to-class="w-0 h-0"
      leave-active-class="duration-[1000ms] ease-out rounded-full"
      @after-enter="onWipeDone"
    >
      <!-- The wipe itself -->
      <div
        v-if="state === 'running' || state === 'done'"
        :class="
          clsx(
            'bg-neutral-50 dark:bg-neutral-900 fixed z-80 translate-x-[-50%] translate-y-[-50%] inset-1/2 w-full h-full overflow-hidden',
          )
        "
        :style="{
          ...(state === 'running' && {
            top: `${origin.y}px`,
            left: `${origin.x}px`,
          }),
        }"
      ></div>
    </Transition>
    <!-- Div for holding wipe content -->
    <div
      :class="clsx('fixed left-0 top-0 z-90 w-screen h-screen overflow-hidden')"
    >
      <div class="flex flex-col items-center justify-center w-full h-full">
        <slot name="afterWipe" :state="state" />
      </div>
    </div>

    <!-- During wipe overlay -->
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
import { ref } from "vue";
import clsx from "clsx";
import defer from "@/utils/defer_promise";

export type WipeState = "idle" | "running" | "done" | "exiting";
type Position = { x: number; y: number };

const state = ref("idle" as WipeState);
const origin = ref<Position>({ x: 0, y: 0 });
let openDonePromise: ReturnType<typeof defer>;

const onWipeDone = () => {
  state.value = "done";
  openDonePromise.resolve();
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

  // Now setting up the wipe itself
  state.value = "running";

  // run the wipe!
  openDonePromise = defer();
  return openDonePromise.promise;
};

const close = () => {
  origin.value = {
    x: window.innerWidth / 2,
    y: window.innerHeight / 2,
  };

  state.value = "exiting";
};

defineExpose({ open, close });
</script>
