<template>
  <div
    class="flex flex-row items-center gap-2.5 px-4 py-4 text-xs relative"
    :class="classes"
  >
    <FuncSkeleton class="flex-shrink-0" />
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ props.name }}
    </div>
    <SiChip
      :text="props.isBuiltin ? 'read-only' : 'custom'"
      :variant="props.isBuiltin ? 'warning' : 'neutral'"
      class="right-4"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import SiChip from "@/atoms/SiChip.vue";
import FuncSkeleton from "@/atoms/FuncSkeleton.vue";

const props = defineProps<{
  color: string;
  name?: string;
  class?: string;
  isBuiltin?: boolean;
}>();

const classes = computed(() => props.class);
</script>
