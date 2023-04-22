<template>
  <div class="w-64 shrink-0 border-shade-100 h-full flex flex-col">
    <!-- Filter button and its dropdown -->
    <div
      class="w-full border-b border-shade-100 p-xs flex items-center justify-between"
    >
      <div class="whitespace-nowrap text-ellipsis overflow-hidden">
        Components Menu
      </div>
      <VButton
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
        v-for="(component, index) in props.componentList"
        :key="component.id"
        v-tooltip="overflowTooltips[index]"
        :class="
          component.id === selectedComponentId
            ? 'bg-action-500'
            : 'hover:bg-black'
        "
        class="py-xs px-xs cursor-pointer flex justify-between items-center leading-tight h-10"
        @click="componentsStore.setSelectedComponentId(component.id)"
      >
        <span
          ref="componentsListRef"
          class="shrink h-full min-w-0 truncate mr-3"
        >
          {{ component.name }}
        </span>
        <slot name="icon" v-bind="{ component }"></slot>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import {
  VButton,
  DropdownMenu,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";

import { useComponentsStore } from "@/store/components.store";
import { ChangeStatus } from "@/api/sdf/dal/change_set";

export interface ComponentListItem {
  id: string;
  name: string;
  schema?: string;
  changeStatus?: ChangeStatus;
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

const componentsListRef = ref();
const overflowTooltips = computed(() => {
  type TooltipInfo = {
    content?: string;
    delay?: { show: number; hide: number };
  };

  const tooltips: TooltipInfo[] = [];

  if (!componentsListRef.value) {
    return tooltips;
  }

  props.componentList.forEach((c) => {
    const el = componentsListRef.value[props.componentList.indexOf(c)];

    if (el !== undefined && el.offsetWidth < el.scrollWidth) {
      tooltips.push({ content: c.name, delay: { show: 700, hide: 10 } });
    } else {
      tooltips.push({});
    }
  });

  return tooltips;
});

// const el = componentsListRef.value[props.componentList.indexOf(component)];
// if (el.offsetWidth < el.scrollWidth) {
//   return { content: component.name, delay: { show: 700, hide: 10 } };
// } else {
//   return {};
// }
</script>
