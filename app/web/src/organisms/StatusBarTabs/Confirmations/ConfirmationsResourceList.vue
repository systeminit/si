<template>
  <div class="w-60 shrink-0 border-shade-100 h-full flex flex-col border-l">
    <!-- Filter button and its dropdown -->
    <div
      class="h-11 w-full border-b border-shade-100 text-lg px-4 flex items-center flex-none"
    >
      <span class="block whitespace-nowrap text-ellipsis overflow-hidden"
        >Resources Menu</span
      >
    </div>
    <!-- List of resources -->
    <div class="overflow-y-auto flex-expand">
      <div
        v-for="resource in resources"
        :key="resource.id"
        :class="
          clsx(
            'py-xs pl-sm pr-xs cursor-pointer flex justify-between items-center leading-tight',
            resource.id === selected ? 'bg-action-500' : 'hover:bg-black',
          )
        "
        @click="emit('select', resource.id)"
      >
        <span class="shrink min-w-0 truncate mr-3">
          {{ resource.name }}
        </span>
        <HealthIcon
          :health="resource.health"
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
import clsx from "clsx";
import HealthIcon from "@/molecules/HealthIcon.vue";
import { MockResource } from "@/store/resources.store";

defineProps({
  resources: {
    type: Array as PropType<MockResource[]>,
    default: () => [],
  },
  selected: { type: Number },
});

const emit = defineEmits<{
  (e: "select", id: number): void;
}>();
</script>
