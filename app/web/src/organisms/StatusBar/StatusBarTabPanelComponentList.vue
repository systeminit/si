<template>
  <div class="w-64 shrink-0 border-shade-100 h-full flex flex-col">
    <!-- Filter button and its dropdown -->
    <div
      class="h-11 w-full border-b border-shade-100 text-lg px-4 flex items-center flex-none"
    >
      <span class="block whitespace-nowrap text-ellipsis overflow-hidden"
        >Components Menu</span
      >
    </div>
    <SiBarButton
      class="h-11 border-b border-shade-100 flex-none"
      tooltip-text="Filter"
      fill-entire-width
    >
      <template #default="{ hovered, open }">
        <div class="flex flex-row items-center">
          {{ selectedFilter.title }}
          <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
        </div>
      </template>

      <template #dropdownContent>
        <SiDropdownItem
          v-for="option of filterOptions"
          :key="option.value"
          :checked="selectedFilter.value === option.value"
          @select="emit('filter', option)"
        >
          {{ option.title }}
        </SiDropdownItem>
      </template>
    </SiBarButton>

    <!-- List of components -->
    <div class="overflow-y-auto flex-expand">
      <div
        v-for="component in props.componentList"
        :key="component.id"
        :class="
          component.id === selectedComponentId
            ? 'bg-action-500'
            : 'hover:bg-black'
        "
        class="py-xs pl-sm pr-xs cursor-pointer flex justify-between items-center leading-tight"
        @click="SelectionService.setSelectedComponentId(component.id)"
      >
        <span class="shrink h-full min-w-0 truncate mr-3">
          {{ component.name }}
        </span>
        <StatusIndicatorIcon
          v-if="component.status"
          :status="component.status"
          class="w-6 shrink-0"
        />
        <HealthIcon
          v-if="component.health"
          :health="component.health"
          hide-text
          remove-right-padding
          size="md"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import StatusIndicatorIcon, {
  Status,
} from "@/molecules/StatusIndicatorIcon.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import { SelectionService } from "@/service/selection";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import HealthIcon, { Health } from "@/molecules/HealthIcon.vue";
import { ComponentType } from "@/service/confirmation";

export interface ComponentListItem {
  id: number;
  name: string;
  type?: ComponentType;
  status?: Status;
  health?: Health;
}

export interface FilterOption {
  value: string;
  title: string;
}

const emit = defineEmits<{
  (e: "filter", filterOption: FilterOption): void;
}>();

const props = defineProps<{
  componentList: ComponentListItem[];
  filterOptions?: FilterOption[];
  selectedFilter?: FilterOption;
}>();

const defaultFilterOption = {
  value: "all",
  title: "Show All",
};

const selectedFilter = computed(() => {
  return props.selectedFilter ?? defaultFilterOption;
});
const filterOptions = computed(() => {
  if (props.filterOptions && props.filterOptions.length > 0) {
    return props.filterOptions;
  }
  return [defaultFilterOption];
});

const selectedComponentId = SelectionService.useSelectedComponentId();
</script>
