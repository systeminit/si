<template>
  <div class="h-[50%] w-full">
    <Icon v-if="show" :size="props.size" name="loader" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Icon, IconSizes } from "@si/vue-lib/design-system";

const props = defineProps<{
  size: IconSizes;
}>();

/**
 * The idea of this component is that, we only want to show a big loader
 * when folks are actually waiting a significant amount of time
 *
 * We don't want to "flash" a loader at them for a few frames bc the tanstack
 * query took ~90ms to return...
 */
const DELAY_MS = 800;
const show = ref(false);
onMounted(() => {
  setTimeout(() => {
    show.value = true;
  }, DELAY_MS);
});
</script>
