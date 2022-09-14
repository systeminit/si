<template>
  <div class="w-60 shrink-0 border-shade-100 h-full flex flex-col border-l">
    <!-- Filter button and its dropdown -->
    <div
      class="h-11 w-full border-b border-shade-100 text-sm px-4 flex items-center flex-none"
    >
      <span class="block whitespace-nowrap text-ellipsis overflow-hidden"
        >Resources For {{ componentName }}</span
      >
    </div>
    <!-- List of resources -->
    <div class="overflow-y-auto flex-expand">
      <div
        v-for="resource in resources"
        :key="resource.id"
        :class="
          resource.id === selectedResourceId
            ? 'bg-action-500'
            : 'hover:bg-black'
        "
        class="py-2 pl-4 pr-3 cursor-pointer flex justify-between items-center"
        @click="selectResource()"
      >
        <span class="shrink min-w-0 truncate mr-3">
          {{ resource.name }}
        </span>
        <HealthIcon
          :status="resource.health"
          hide-text
          remove-right-padding
          size="md"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import HealthIcon, { Health } from "@/molecules/HealthIcon.vue";

export type Resource = {
  id: number;
  name: string;
  health: Health;
};

defineProps({
  resources: {
    type: Array as PropType<Resource[]>,
    default: () => [],
  },
  componentName: { type: String, required: true },
});

const selectedResourceId = 0;

const selectResource = () => {
  console.log("NOT DONE YET!");
};
</script>
