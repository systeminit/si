<template>
  <div class="flex flex-row w-full h-full bg-transparent overflow-hidden">
    <SiPanel
      remember-size-key="workflow-left"
      side="left"
      size-classes="w-96 flex-none"
    >
      <WorkflowPicker
        :list="list"
        :selected-id="selected?.id ?? null"
        @selected="select"
      />
    </SiPanel>
    <div
      class="grow overflow-x-hidden overflow-y-hidden dark:bg-neutral-800 dark:text-white text-lg font-semi-bold px-2 pt-2 flex flex-col"
    >
      <span v-if="selected">
        <WorkflowResolver :selected-id="selected.id">
          <template #runButton>
            <VButton
              icon="play"
              label="Run"
              size="lg"
              class="w-48"
              @click="runWorkflow()"
            />
          </template>
        </WorkflowResolver>
      </span>
      <div
        v-else
        class="p-2 text-center text-neutral-400 dark:text-neutral-300"
      >
        Select a workflow to resolve.
      </div>
    </div>
    <SiPanel
      remember-size-key="workflow-right"
      side="right"
      size-classes="w-96 flex-none"
    >
      <div class="p-2 w-full h-full overflow-hidden">
        <CodeViewer v-if="logs" :code="logs.join('\n')">
          <template #title>Output</template>
        </CodeViewer>
        <div v-else class="p-2 text-neutral-400 dark:text-neutral-300">
          When you run a workflow, it's output will display here.
        </div>
      </div>
    </SiPanel>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { refFrom, untilUnmounted } from "vuse-rx";
import WorkflowPicker from "@/organisms/WorkflowRunner/WorkflowPicker.vue";
import WorkflowResolver from "@/organisms/WorkflowRunner/WorkflowResolver.vue";
import { WorkflowService } from "@/service/workflow";
import { eventCommandOutput$ } from "@/observable/command";
import {
  ListedWorkflowView,
  ListWorkflowsResponse,
} from "@/service/workflow/list";
import VButton from "@/molecules/VButton.vue";
import SiPanel from "@/atoms/SiPanel.vue";
import CodeViewer from "@/organisms/CodeViewer.vue";

const selected = ref<ListedWorkflowView | null>(null);
const select = (w: ListedWorkflowView | null) => {
  logs.value = null;
  selected.value = w;
};

const logs = ref<string[] | null>(null);
const runWorkflow = async () => {
  if (selected.value) {
    logs.value = null;
    const outputs = await WorkflowService.run({ id: selected.value.id });
    logs.value = outputs?.logs ?? null;
  }
};

eventCommandOutput$.pipe(untilUnmounted).subscribe((command) => {
  if (!command) return;
  if (!logs.value) logs.value = [];
  logs.value.push(command.payload.data.output);
});

const list = refFrom<ListWorkflowsResponse>(WorkflowService.list(), []);
</script>
