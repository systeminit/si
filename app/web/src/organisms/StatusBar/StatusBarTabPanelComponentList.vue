<template>
  <div class="w-64 shrink-0 border-shade-100 h-full flex flex-col">
    <!-- Filter button and its dropdown -->
    <div
      class="w-full border-b border-shade-100 p-xs flex items-center justify-between"
    >
      <div class="whitespace-nowrap text-ellipsis overflow-hidden">
        Components Menu
      </div>
      <VButton2
        icon="filter"
        size="sm"
        class="justify-self-end"
        variant="ghost"
        @click="filterMenuRef?.open"
      />

      <DropdownMenu ref="filterMenuRef">
        <DropdownMenuItem
          v-for="option of filterOptions"
          :key="option.value"
          :checked="selectedFilter.value === option.value"
          @select="emit('filter', option)"
        >
          {{ option.title }}
        </DropdownMenuItem>
      </DropdownMenu>
    </div>

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
        class="py-xs px-xs cursor-pointer flex justify-between items-center leading-tight"
        @click="componentsStore.setSelectedComponentId(component.id)"
      >
        <span class="shrink h-full min-w-0 truncate mr-3">
          {{ component.name }}
        </span>
        <slot name="icon" v-bind="{ component }"></slot>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useComponentsStore } from "@/store/components.store";
import VButton2 from "@/ui-lib/VButton2.vue";
import DropdownMenu from "@/ui-lib/menus/DropdownMenu.vue";
import DropdownMenuItem from "@/ui-lib/menus/DropdownMenuItem.vue";

export interface ComponentListItem {
  id: number;
  name: string;
  schema?: string;
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

const filterMenuRef = ref<InstanceType<typeof DropdownMenu>>();

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

const componentsStore = useComponentsStore();
const selectedComponentId = computed(() => componentsStore.selectedComponentId);
</script>
