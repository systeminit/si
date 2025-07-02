<template>
  <div v-if="show">
    <ExploreGridSkeleton />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import ExploreGridSkeleton from "./ExploreGridSkeleton.vue";

/**
 * The idea of this component is that, we only want to show a big loader
 * when folks are actually waiting a significant amount of time
 *
 * We don't want to "flash" a loader at them for a few frames bc the tanstack
 * query took ~90ms to return...
 */
const DELAY = 300; // 300ms
const show = ref(false);
onMounted(() => {
  setTimeout(() => {
    show.value = true;
  }, DELAY);
});
</script>
