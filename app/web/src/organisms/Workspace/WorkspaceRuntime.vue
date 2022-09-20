<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="220">
    <WorkflowPicker
      :list="workflowList"
      :selected-id="selected?.id ?? null"
      @selected="select"
    />
  </SiPanel>
  <div
    class="grow overflow-hidden h-full bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 text-lg font-semi-bold px-2 pt-2 flex flex-col"
  >
    <WorkflowResolver v-if="selected" :selected="selected">
      <template #runButton>
        <VButton
          icon="play"
          label="Run"
          size="lg"
          class="w-48"
          button-type="success"
          @click="runWorkflow()"
        />
      </template>
    </WorkflowResolver>

    <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
      Select a workflow to resolve.
    </div>
  </div>
  <SiPanel remember-size-key="workflow-right" side="right">
    <SiTabGroup v-if="logs" :selected-index="0">
      <template #tabs>
        <SiTabHeader>Output</SiTabHeader>
        <SiTabHeader>Resources</SiTabHeader>
      </template>
      <template #panels>
        <TabPanel class="h-full p-xs overflow-hidden">
          <WorkflowOutput :logs="logs" :status="currentWorkflowStatus" />
        </TabPanel>
        <TabPanel>
          <WorkflowResources :components="componentsList" />
        </TabPanel>
      </template>
    </SiTabGroup>

    <div v-else class="p-4 text-neutral-400 dark:text-neutral-300">
      {{
        selected
          ? `When you run ${selected.title}, the output will display here.`
          : "When you run a workflow, the output will display here."
      }}
    </div>
  </SiPanel>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { refFrom, untilUnmounted } from "vuse-rx";
import { TabPanel } from "@headlessui/vue";
import WorkflowPicker from "@/organisms/WorkflowRunner/WorkflowPicker.vue";
import WorkflowResolver from "@/organisms/WorkflowRunner/WorkflowResolver.vue";
import { WorkflowService } from "@/service/workflow";
import { eventCommandOutput$, eventCommandReturn$ } from "@/observable/command";
import {
  ListedWorkflowView,

  ListWorkflowsResponse,
} from "@/service/workflow/list";
import VButton from "@/molecules/VButton.vue";
import SiPanel from "@/atoms/SiPanel.vue";
import { WorkflowStatus } from "@/molecules/WorkflowStatusIcon.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { ConfirmationService } from "@/service/confirmation";
import WorkflowOutput from "../WorkflowRunner/WorkflowOutput.vue";
import WorkflowResources from "../WorkflowRunner/WorkflowResources.vue";
import { ComponentListItem } from "../StatusBar/StatusBarTabPanelComponentList.vue";

const runId = ref<null | number>(null);

const selected = ref<{
  id: number;
  title: string;
  componentId: number | null;
} | null>(null);
const select = (w: ListedWorkflowView, componentId: number | null) => {
  logs.value = null;
  runId.value = null;
  selected.value = { id: w.id, title: w.title, componentId };
};

const logs = ref<string[] | null>(null);
const runWorkflow = async () => {
  if (selected.value) {
    logs.value = null;
    runId.value = null;
    currentWorkflowStatus.value = "running";
    const response = await WorkflowService.run({
      id: selected.value.id,
      componentId: selected.value.componentId,
    });
    if (response) {
      runId.value = response.runId;
    }
  }
};

const currentWorkflowStatus = ref("running" as WorkflowStatus);

eventCommandOutput$.pipe(untilUnmounted).subscribe((command) => {
  if (!command) return;
  if (runId.value !== command.payload.data.runId) return;
  if (!logs.value) logs.value = [];
  logs.value.push(command.payload.data.output);
});

eventCommandReturn$.pipe(untilUnmounted).subscribe((command) => {
  if (!command) return;
  if (runId.value !== command.payload.data.runId) return;
  if (!logs.value) logs.value = [];
  logs.value = command.payload.data.output;
  currentWorkflowStatus.value = command.payload.data.runnerState.status;
});

const workflowList = refFrom<ListWorkflowsResponse>(WorkflowService.list(), []);

const confirmationsSummary = ConfirmationService.useConfirmationSummary();

const componentsList = computed((): ComponentListItem[] => {
  if (confirmationsSummary.value === undefined) return [];
  const list: ComponentListItem[] = [];
  for (const component of confirmationsSummary.value.components) {
    list.push({
      id: component.id,
      name: component.name,
      type: component.type,
      health: component.health,
    });
  }
  return list;
});
</script>
