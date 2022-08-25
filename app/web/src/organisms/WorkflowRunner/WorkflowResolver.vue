<template>
  <CodeViewer
    :code="workflowTree?.json ?? '// Resolving Workflow'"
    code-language="json"
  >
    <template #title>
      <span class="text-lg ml-4">
        {{ JSON.parse(workflowTree?.json ?? "{}")?.name }} Plan
      </span>
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
