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
    <!-- List of components -->
    <div class="overflow-y-auto flex-expand">
      <div
        v-for="componentId in componentIds"
        :key="componentId"
        :class="
          clsx(
            'py-xs pl-sm pr-xs cursor-pointer flex justify-between items-center leading-tight',
            componentId === selectedComponentId
              ? 'bg-action-500'
              : 'hover:bg-black',
          )
        "
        @click="onSelectResource(componentId)"
      >
        <span class="shrink min-w-0 truncate mr-3">
          {{ componentsStore.componentsById[componentId]?.displayName }}
        </span>
        <StatusIndicatorIcon
          type="confirmation"
          :status="fixesStore.confirmationStatusByComponentId[componentId]"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import clsx from "clsx";
import { useFixesStore } from "@/store/fixes.store";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { ComponentId, useComponentsStore } from "@/store/components.store";

const fixesStore = useFixesStore();
const componentsStore = useComponentsStore();
const componentIds = computed(() => fixesStore.allComponents);
const selectedComponentId = computed(() => componentsStore.selectedComponentId);

function onSelectResource(id: ComponentId) {
  componentsStore.setSelectedComponentId(id);
}
</script>
