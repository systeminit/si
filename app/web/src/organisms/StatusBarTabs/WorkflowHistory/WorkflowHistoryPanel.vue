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
    <div class="grow flex flex-col overflow-hidden">
      <div class="h-12 m-2">
        <!-- TODO(wendy) replace fixed info with updating info -->
        <WorkflowStatusBar
          name="workflow-name"
          status="failure"
          timestamp="Thu, 31 Mar 2022 04:20:00 +0000"
        />
      </div>
      <div class="w-full grow flex flex-row overflow-hidden">
        <div
          class="w-1/2 m-2 p-2 border border-neutral-600 rounded overflow-hidden"
        >
          <!-- TODO(wendy) - replace fixed logs/status with updating info -->
          <WorkflowOutput
            :logs="[
              'is this would be log stuff',
              'right now it just nonsense',
              'test',
              'yeah it works',
              'real log stuff coming soon',
              'is this would be log stuff',
              'right now it just nonsense',
              'test',
              'yeah it works',
              'real log stuff coming soon',
              'is this would be log stuff',
              'right now it just nonsense',
              'test',
              'yeah it works',
              'real log stuff coming soon',
            ]"
            status="failure"
          />
        </div>
        <div
          class="w-1/2 my-2 flex flex-col mr-2 border border-neutral-600 rounded overflow-hidden"
        >
          <div class="flex-none border-b p-2 border-neutral-600">
            Resources Impacted
          </div>
          <div class="w-full grow overflow-auto">
            <ul class="list-disc list-inside p-2">
              <li>Resource 0</li>
              <li>Resource 1</li>
              <li>Resource 2</li>
              <li>Resource 3</li>
              <li>Resource 4</li>
              <li>Resource 5</li>
              <li>Resource 6</li>
              <li>Resource 7</li>
              <li>Resource 8</li>
              <li>Resource 9</li>
            </ul>
          </div>
        </div>
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
import WorkflowStatusBar from "@/molecules/WorkflowStatusBar.vue";

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
