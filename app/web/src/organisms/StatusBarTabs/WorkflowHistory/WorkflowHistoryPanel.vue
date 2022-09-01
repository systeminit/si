<template>
  <div class="w-full h-full flex flex-row">
    <div class="w-64 shrink-0 border-shade-100 h-full flex flex-col">
      <span
        class="h-11 border-b border-shade-100 text-lg px-4 flex items-center"
      >
        Workflows
      </span>

      <!-- Filter button and its dropdown -->
      <!-- <SiBarButton
        class="h-11 border-b border-shade-100"
        tooltip-text="Sort"
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
      </SiBarButton> -->

      <!-- List of workflows -->
      <div class="overflow-y-auto flex-expand">
        <div
          v-for="workflow in workflowList"
          :key="workflow.id"
          :class="
            workflow.id === selectedWorkflowId
              ? 'bg-action-500'
              : 'hover:bg-black'
          "
          class="py-2 pl-4 pr-3 cursor-pointer flex flex-row items-center"
          @click="selectWorkflow(workflow)"
        >
          <WorkflowStatusIcon :status="workflow.status" />
          <span class="shrink min-w-0 truncate mr-3 whitespace-nowrap">
            {{ workflow.title }}
          </span>
        </div>
      </div>
    </div>
    <!-- Currently selected Workflow info panel -->
    <div v-if="selectedWorkflowInfo" class="grow flex flex-col overflow-hidden">
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
            :logs="selectedWorkflowInfo.logs"
            :status="selectedWorkflowInfo.status"
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
import { ref } from "vue";
import { refFrom, fromRef } from "vuse-rx/src";
// import SiBarButton from "@/molecules/SiBarButton.vue";
// import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
// import SiArrow from "@/atoms/SiArrow.vue";
import { combineLatest, from, switchMap } from "rxjs";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";
import WorkflowOutput from "@/organisms/WorkflowRunner/WorkflowOutput.vue";
import WorkflowStatusBar from "@/molecules/WorkflowStatusBar.vue";
import { WorkflowService } from "@/service/workflow";
import {
  ListedWorkflowHistoryView,
  ListWorkflowsHistoryResponse,
} from "@/service/workflow/history";
import { WorkflowRunInfo } from "@/service/workflow/info";

export interface FilterOption {
  value: string;
  title: string;
}

const props = defineProps<{
  filterOptions?: FilterOption[];
  selectedFilter?: FilterOption;
}>();

const selectedWorkflowId = ref<number | null>(null);

const selectedWorkflowId$ = fromRef<number | null>(selectedWorkflowId, {
  immediate: true,
});

// const defaultFilterOption = {
//   value: "r",
//   title: "Most Recent",
// };

// const selectedFilter = computed(() => {
//   return props.selectedFilter ?? defaultFilterOption;
// });

// const emit = defineEmits<{
//   (e: "filter", filterOption: FilterOption): void;
// }>();

const selectWorkflow = (workflow: ListedWorkflowHistoryView) => {
  selectedWorkflowId.value = workflow.id;
};

const workflowList = refFrom<ListWorkflowsHistoryResponse>(
  WorkflowService.history(),
  [],
);

const selectedWorkflowInfo = refFrom<WorkflowRunInfo | null>(
  combineLatest([selectedWorkflowId$]).pipe(
    switchMap(([id]) => {
      if (!id) return from([null]);

      return WorkflowService.info({ id });
    }),
  ),
);
</script>
