<template>
  <!-- Do not include special case management funcs, like "import" and "run template" -->
  <li
    v-if="
      func.kind !== MgmtFuncKind.Import &&
      func.kind !== MgmtFuncKind.RunTemplate
    "
    :class="
      clsx(
        'rounded border flex flex-col',
        themeClasses('border-neutral-400', 'border-neutral-600'),
      )
    "
  >
    <div
      :class="
        clsx(
          'border-b w-full p-xs text-sm',
          themeClasses('border-neutral-400', 'border-neutral-600'),
        )
      "
    >
      {{ func.name }}
    </div>
    <div class="w-full flex place-content-between items-center p-sm">
      <span>
        {{ func.description ?? "No description provided" }}
      </span>

      <VButton
        size="sm"
        label="Run Function"
        iconTone="action"
        loadingText="Running"
        :loading="managementExecutionStatus === 'Running'"
        @click="runMgmtFunc(func.id)"
      />
    </div>
    <div
      v-if="
        funcRun &&
        managementExecutionStatusIcon &&
        managementExecutionStatusMessage
      "
      :class="
        clsx(
          'm-sm flex items-center gap-xs px-xs py-sm rounded',
          themeClasses('bg-neutral-300', 'bg-neutral-700'),
        )
      "
    >
      <Icon :name="managementExecutionStatusIcon" class="shrink-0" />
      <span class="grow">
        {{ managementExecutionStatusMessage }}
      </span>
      <VButton
        size="sm"
        :label="seeFuncRunLabel"
        iconTone="action"
        class="shrink-0"
        @click="navigateToFuncRunDetails(funcRun.id)"
      />
    </div>
  </li>
</template>

<script setup lang="ts">
import {
  Icon,
  IconNames,
  themeClasses,
  VButton,
} from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import { computed, inject, ref } from "vue";
import clsx from "clsx";
import { routes, useApi } from "@/newhotness/api_composables";
import { funcRunStatus, FuncRun } from "@/newhotness/api_composables/func_run";
import { MgmtFuncKind, MgmtFunction } from "@/workers/types/entity_kind_types";
import { Context, assertIsDefined } from "@/newhotness/types";
import { useManagementFuncJobState } from "./logic_composables/management";

const props = defineProps<{
  componentId: string;
  func: MgmtFunction;
  funcRun?: FuncRun;
}>();

const seeFuncRunLabel = "See Func Run";

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const router = useRouter();
const route = useRoute();

const mgmtRunApi = useApi();
const runMgmtFunc = async (funcId: string) => {
  const call = mgmtRunApi.endpoint<{ success: boolean }>(routes.MgmtFuncRun, {
    prototypeId: funcId,
    componentId: props.componentId,
    viewId: "DEFAULT",
  });

  const { req, newChangeSetId } = await call.post({});

  dispatchedFunc.value = true;
  setTimeout(() => {
    dispatchedFunc.value = false;
  }, 2000);

  // NOTE(nick): need to make sure this makes sense after the timeout.
  if (mgmtRunApi.ok(req) && newChangeSetId) {
    mgmtRunApi.navigateToNewChangeSet(
      {
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.componentId,
        },
      },
      newChangeSetId,
    );
  }
};

const dispatchedFunc = ref(false);

const funcRun = computed(() => props.funcRun);
const managementFuncJobStateComposable = useManagementFuncJobState(funcRun);
const managementFuncJobState = computed(
  () => managementFuncJobStateComposable.value.value,
);
const managementExecutionStatus = computed(() => {
  if (!props.funcRun) return undefined;
  if (dispatchedFunc.value) return "Running";
  return funcRunStatus(props.funcRun, managementFuncJobState.value?.state);
});

const managementExecutionStatusIcon = computed((): IconNames | undefined => {
  if (managementExecutionStatus.value === "Success") return "check-circle";
  if (managementExecutionStatus.value === "Failure") return "x-circle";
  if (managementExecutionStatus.value === "Running") return "loader";
  return undefined;
});

const managementExecutionStatusMessage = computed((): string | undefined => {
  if (managementExecutionStatus.value === "Success")
    return `Executed successfully. Click "${seeFuncRunLabel}" see execution results.`;
  if (managementExecutionStatus.value === "Failure")
    return `Error executing function. Click "${seeFuncRunLabel}" to get more details.`;
  if (managementExecutionStatus.value === "Running")
    return "Running Function...";
  return undefined;
});

const navigateToFuncRunDetails = (funcRunId: string) => {
  router.push({
    name: "new-hotness-func-run",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      funcRunId,
    },
  });
};
</script>
