<template>
  <div
    class="w-0.5 h-full bg-neutral-300 dark:bg-neutral-600 cursor-col-resize z-30 relative"
  >
    <div
      class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] w-3 h-16 rounded-full bg-neutral-200 dark:bg-neutral-700 border border-neutral-300 dark:border-neutral-900 text-xl"
    >
      <DotsVerticalIcon
        class="absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] w-6 text-neutral-400 dark:text-neutral-500"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { DotsVerticalIcon } from "@heroicons/vue/solid";
</script>
