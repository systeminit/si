<template>
  <div class="w-full h-full flex flex-row">
    <div class="w-64 shrink-0 border-shade-100 h-full flex flex-col">
      <!-- Filter button and its dropdown -->
      <span
        class="h-11 border-b border-shade-100 text-lg px-4 flex items-center"
      >
        Workflows
      </span>
      <SiBarButton
        class="h-11 border-b border-shade-100"
        tooltip-text="Filter"
        fill-entire-width
      >
        <template #default="{ hovered, open }">
          <div class="flex flex-row">
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

      <!-- List of workflows -->
      <div class="overflow-y-auto flex-expand">
        <div
          v-for="workflow in props.workflows"
          :key="workflow.id"
          :class="
            workflow.id === selectedWorkflowId
              ? 'bg-action-500'
              : 'hover:bg-black'
          "
          class="py-2 pl-4 pr-3 cursor-pointer flex flex-row items-center"
          @click="selectWorkflow(workflow)"
        >
          <WorkflowStatusIcon status="running" />
          <span class="shrink min-w-0 truncate mr-3 whitespace-nowrap">
            {{ workflow.name }}
          </span>
        </div>
      </div>
    </div>
    <div class="grow h-full flex flex-col">
      <div class="h-12 text-center">Workflow Status Bar Goes Here</div>
      <div class="w-full grow flex flex-row">
        <div class="w-1/2 p-2">
          <WorkflowOutput
            :logs="[
              'is this would be log stuff',
              'right now it just nonsense',
              'test',
              'yeah it works',
              'real log stuff coming soon',
            ]"
            status="success"
          />
        </div>
        <div class="w-1/2 p-2 text-center">Resources Impacted Goes Here</div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";
import WorkflowOutput from "@/organisms/WorkflowRunner/WorkflowOutput.vue";

export type WorkflowInfo = {
  id: number;
  name: string;
};

export interface FilterOption {
  value: string;
  title: string;
}

const defaultFilterOption = {
  value: "all",
  title: "Show All",
};

const props = defineProps<{
  workflows: WorkflowInfo[];
  filterOptions?: FilterOption[];
  selectedFilter?: FilterOption;
}>();

const selectedWorkflowId = 1;

const selectedFilter = computed(() => {
  return props.selectedFilter ?? defaultFilterOption;
});

const emit = defineEmits<{
  (e: "filter", filterOption: FilterOption): void;
}>();

const selectWorkflow = (workflow: WorkflowInfo) => {
  console.log(workflow);
  console.log("TODO(wendy) - write function to select workflow here!");
};
</script>
