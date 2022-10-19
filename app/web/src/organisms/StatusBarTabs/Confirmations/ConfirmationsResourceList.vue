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
            resource.componentId === selectedComponentId
              ? 'bg-action-500'
              : 'hover:bg-black',
          )
        "
        @click="onSelectResource(resource.componentId)"
      >
        <span class="shrink min-w-0 truncate mr-3">
          {{ resource.name }}
        </span>
        <StatusIndicatorIcon
          type="confirmation"
          :status="
            resourcesStore.confirmationStatusByComponentId[resource.componentId]
          "
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import clsx from "clsx";
import { useResourcesStore } from "@/store/resources.store";
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";
import { ComponentId, useComponentsStore } from "@/store/components.store";

const resourcesStore = useResourcesStore();
const componentsStore = useComponentsStore();
const resources = computed(() => resourcesStore.allResources);
const selectedComponentId = computed(() => componentsStore.selectedComponentId);

function onSelectResource(id: ComponentId) {
  componentsStore.setSelectedComponentId(id);
}
</script>
