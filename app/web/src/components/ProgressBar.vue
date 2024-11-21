<template>
  <div class="w-full h-4 bg-neutral-300 rounded-md">
    <!-- min-w needs to be Arbitrary because tailwind does not allow the usual sizing (*-1, *-2) on the min-w class -->
    <div
      class="h-full text-white bg-success-600 rounded-md min-w-[0.5rem] max-w-full transition-[width] duration-500"
      :style="{ width: `${progressPercent * 100}%` }"
    />
  </div>
</template>

<script setup lang="ts">
const props = defineProps({
  // Should be between 0 and 1
  progressPercent: { type: Number, default: 0 },
});
</script>
