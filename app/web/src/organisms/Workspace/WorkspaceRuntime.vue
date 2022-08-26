<template>
  <div class="flex flex-row w-full h-full bg-transparent overflow-hidden">
    <SiSidebar side="left" class="h-full pb-12" width-classes="shrink-0 w-96">
      <WorkflowPicker
        :list="list"
        :selected-id="selected?.id ?? null"
        @selected="select"
      />
    </SiSidebar>
    <div
      class="grow overflow-x-hidden overflow-y-hidden dark:bg-neutral-800 dark:text-white text-lg font-semi-bold px-2 pt-2 flex flex-col"
    >
      <span v-if="selected">
        <div class="w-full flex flex-row-reverse">
          <VButton
            icon="play"
            label="Run"
            size="lg"
            class="w-48"
            @click="runWorkflow()"
          />
        </div>
        <WorkflowResolver :selected-id="selected.id" />
      </span>
      <div
        v-else
        class="p-2 text-center text-neutral-400 dark:text-neutral-300"
      >
        Select a workflow to resolve.
      </div>
    </div>
    <SiSidebar
      :hidden="false"
      side="right"
      class="h-full"
      width-classes="shrink-0 w-80"
    >
      <span v-if="logs" class="overflow-auto">
        <p class="text-lg">Output:</p>
        <p v-for="(log, index) in logs" :key="index">{{ log }}</p>
      </span>
      <!-- if hiding is added later, condition is selectedFuncId < 1 -->
      <!--<FuncDetails :func-id="selectedFunc.id" />-->
    </SiSidebar>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { refFrom } from "vuse-rx/src";
import SiSidebar from "@/atoms/SiSidebar.vue";
import WorkflowPicker from "@/organisms/WorkflowRunner/WorkflowPicker.vue";
import WorkflowResolver from "@/organisms/WorkflowRunner/WorkflowResolver.vue";
import { WorkflowService } from "@/service/workflow";
import {
  ListedWorkflowView,
  ListWorkflowsResponse,
} from "@/service/workflow/list";
import VButton from "@/molecules/VButton.vue";

const selected = ref<ListedWorkflowView | null>(null);
const select = (w: ListedWorkflowView | null) => {
  logs.value = null;
  selected.value = w;
};

const logs = ref<string[] | null>(null);
const runWorkflow = async () => {
  if (selected.value) {
    const outputs = await WorkflowService.run({ id: selected.value.id });
    logs.value = outputs?.logs ?? null;
  }
};

const list = refFrom<ListWorkflowsResponse>(WorkflowService.list(), []);
</script>
