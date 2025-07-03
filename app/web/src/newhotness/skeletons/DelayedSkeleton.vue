<template>
  <div v-if="show">
    <ExploreGridSkeleton v-if="skeleton === 'grid'" />
    <ExploreMapSkeleton v-if="skeleton === 'map'" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import ExploreGridSkeleton from "./ExploreGridSkeleton.vue";
import ExploreMapSkeleton from "./ExploreMapSkeleton.vue";

defineProps<{
  skeleton: "map" | "grid";
}>();

const DELAY = 300; // 300ms
const show = ref(false);
onMounted(() => {
  setTimeout(() => {
    show.value = true;
  }, DELAY);
});
</script>

<style lang="css">
.skeleton-shimmer {
  position: relative;
  overflow: hidden;
}

.skeleton-shimmer::before {
  content: "";
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  animation: shimmer 1.5s infinite;
}

/* Light theme shimmer */
body.light .skeleton-shimmer::before {
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.8),
    transparent
  );
}

/* Dark theme shimmer */
body.dark .skeleton-shimmer::before {
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.1),
    transparent
  );
}

@keyframes shimmer {
  0% {
    left: -100%;
  }
  100% {
    left: 100%;
  }
}
</style>
