<template>
  <div class="w-full h-full flex flex-row">
    <div class="w-64 shrink-0 border-shade-100 h-full flex flex-col">
      <span
        class="h-11 border-b border-shade-100 text-lg px-4 flex items-center flex-none"
      >
        Workflows Menu
      </span>

      <!-- Sort button and its dropdown -->
      <SiBarButton
        class="h-11 border-b border-shade-100 flex-none"
        tooltip-text="Sort"
        fill-entire-width
      >
        <template #default="{ hovered, open }">
          <div class="flex flex-row items-center">
            {{ selectedSort.title }}
            <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
          </div>
        </template>

        <template #dropdownContent>
          <SiDropdownItem
            v-for="option of sortOptions"
            :key="option.value"
            :checked="selectedSort.value === option.value"
            @select="emit('sort', option)"
          >
            {{ option.title }}
          </SiDropdownItem>
        </template>
      </SiBarButton>

      <!-- List of workflows -->
      <div class="overflow-y-auto flex-expand">
        <div
          v-for="workflow in workflowListDisplay"
          :key="workflow.id"
          :class="
            workflow.id === selectedWorkflowId
              ? 'bg-action-500'
              : 'hover:bg-black'
          "
          class="py-2 pl-4 pr-3 cursor-pointer flex flex-row items-center leading-tight"
          @click="selectWorkflow(workflow)"
        >
          <WorkflowStatusIcon :status="workflow.status" />
          <span class="truncate mr-3 whitespace-nowrap">
            {{ workflow.title }}
          </span>
          <span class="text-xs text-neutral-400 whitespace-nowrap ml-auto">
            <Timestamp
              :date="new Date(workflow.created_at)"
              size="mini"
              relative
            />
          </span>
        </div>
      </div>
    </div>
    <!-- Currently selected Workflow info panel -->
    <div
      v-if="selectedWorkflowInfo"
      class="grow flex flex-col overflow-hidden bg-shade-100"
    >
      <div class="h-12 mx-2 mt-2">
        <WorkflowStatusBar
          :name="selectedWorkflowInfo.title"
          :status="selectedWorkflowInfo.status"
          :timestamp="selectedWorkflowInfo.created_at"
        />
      </div>
      <div class="w-full grow overflow-x-hidden flex flex-row flex-wrap p-1">
        <div
          class="h-fit max-h-full lg:basis-1/2 grow overflow-hidden flex flex-row p-1"
        >
          <div
            class="w-full shrink grow bg-neutral-800 rounded p-1 flex flex-row"
          >
            <WorkflowOutput
              :logs="selectedWorkflowInfo.logs"
              :status="selectedWorkflowInfo.status"
              force-theme="dark"
            />
          </div>
        </div>
        <div
          class="h-fit max-h-full lg:basis-1/2 grow overflow-hidden flex flex-row p-1"
        >
          <div
            class="w-full shrink grow bg-neutral-800 rounded flex flex-col overflow-auto"
          >
            <WorkflowResources
              force-theme="dark"
              :components="componentsList"
            />
          </div>
        </div>
      </div>
    </div>
    <div
      v-else
      class="grow flex flex-row overflow-hidden bg-shade-100 items-center text-center"
    >
      <p class="w-full text-3xl text-neutral-500">No Workflow Selected</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { refFrom, fromRef } from "vuse-rx/src";
import { combineLatest, from, switchMap } from "rxjs";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";
import WorkflowOutput from "@/organisms/WorkflowRunner/WorkflowOutput.vue";
import WorkflowStatusBar from "@/molecules/WorkflowStatusBar.vue";
import { WorkflowService } from "@/service/workflow";
import {
  ListedWorkflowHistoryView,
  ListWorkflowsHistoryResponse,
} from "@/service/workflow/history";
import { WorkflowRunInfo } from "@/service/workflow/info";
import Timestamp from "@/ui-lib/Timestamp.vue";
import WorkflowResources from "@/organisms/WorkflowRunner/WorkflowResources.vue";
import { ComponentListItem } from "@/organisms/StatusBar/StatusBarTabPanelComponentList.vue";
import { ResourceService } from "@/service/resource";

export interface SortOption {
  value: string;
  title: string;
}

const props = defineProps<{
  sortOptions?: SortOption[];
  selectedSort?: SortOption;
}>();

const selectedWorkflowId = ref<number | null>(null);

const selectedWorkflowId$ = fromRef<number | null>(selectedWorkflowId, {
  immediate: true,
});

const emit = defineEmits<{
  (e: "sort", sortOption: SortOption): void;
}>();

const defaultSortOption = {
  value: "r",
  title: "Newest",
};

const selectedSort = computed(() => {
  return props.selectedSort ?? defaultSortOption;
});

const selectWorkflow = (workflow: ListedWorkflowHistoryView) => {
  selectedWorkflowId.value = workflow.id;
};

const workflowList = refFrom<ListWorkflowsHistoryResponse>(
  WorkflowService.history(),
  [],
);

const workflowListDisplay = computed(() => {
  if (selectedSort.value.value === "r") {
    return [...workflowList.value].reverse();
  } else return workflowList.value;
});

const selectedWorkflowInfo = refFrom<WorkflowRunInfo | null>(
  combineLatest([selectedWorkflowId$]).pipe(
    switchMap(([id]) => {
      if (!id) return from([null]);

      return WorkflowService.info({ id });
    }),
  ),
);

const resourceSummary = ResourceService.useResourceSummary();

const componentsList = computed((): ComponentListItem[] => {
  if (resourceSummary.value === undefined) return [];
  const list: ComponentListItem[] = [];
  for (const component of resourceSummary.value.components) {
    list.push({
      id: component.id,
      name: component.name,
      schema: component.schema,
      health: component.health,
      resource: component.resource,
    });
  }
  return list;
});
</script>
