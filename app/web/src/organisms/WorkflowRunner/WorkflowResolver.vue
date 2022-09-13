<template>
  <CodeViewer
    :code="workflowTree?.json ?? '// Resolving Workflow'"
    code-language="json"
    title-classes="h-12"
    border
  >
    <template #title>
      <div class="text-lg ml-4 whitespace-nowrap text-ellipsis overflow-hidden">
        {{ JSON.parse(workflowTree?.json ?? "{}")?.name }} Plan
      </div>
      <div class="px-2">
        <slot name="runButton" />
      </div>
    </template>
  </CodeViewer>
</template>

<script lang="ts" setup>
import { toRef } from "vue";
import { fromRef, refFrom } from "vuse-rx";
import { combineLatest, switchMap } from "rxjs";
import { WorkflowService } from "@/service/workflow";
import { WorkflowResolveResponse } from "@/service/workflow/resolve";
import CodeViewer from "@/organisms/CodeViewer.vue";

const props = defineProps<{
  selectedId: number;
}>();

const selectedId = toRef(props, "selectedId", 0);
const selectedId$ = fromRef(selectedId, { immediate: true });

const workflowTree = refFrom<WorkflowResolveResponse | null>(
  combineLatest([selectedId$]).pipe(
    switchMap(([id]) => {
      return WorkflowService.resolve({ id });
    }),
  ),
);
</script>
