<template>
  <!-- Do not include special case management funcs, like "import" and "run template" -->
  <li
    v-if="
      func.kind !== MgmtFuncKind.Import &&
      func.kind !== MgmtFuncKind.RunTemplate
    "
  >
    <StatusBox
      :kind="statusBoxKind"
      :text="func.name"
      :description="func.description ?? 'No description provided'"
    >
      <template #right>
        <div class="flex gap-xs">
          <VButton
            size="sm"
            :label="
              managementExecutionStatus === 'Failure'
                ? 'Re-run function'
                : 'Run function'
            "
            :loading="managementExecutionStatus === 'Running'"
            loadingText="Running function"
            :disabled="managementExecutionStatus === 'Running'"
            loadingIcon="loader"
            :class="
              clsx(
                '!text-sm !border !cursor-pointer !px-xs',
                themeClasses(
                  '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                  '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                ),
              )
            "
            @click.stop="runMgmtFunc(func.id)"
          />
          <VButton
            v-if="funcRun"
            size="sm"
            label="See Func Run"
            tone="neutral"
            @click="navigateToFuncRunDetails(funcRun.id)"
          />
        </div>
      </template>
    </StatusBox>
  </li>
</template>

<script setup lang="ts">
import { themeClasses, VButton } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import { computed, inject, ref } from "vue";
import clsx from "clsx";
import { routes, useApi } from "@/newhotness/api_composables";
import { funcRunStatus, FuncRun } from "@/newhotness/api_composables/func_run";
import { MgmtFuncKind, MgmtFunction } from "@/workers/types/entity_kind_types";
import { Context, assertIsDefined } from "@/newhotness/types";
import { useManagementFuncJobState } from "./logic_composables/management";
import StatusBox from "./layout_components/StatusBox.vue";

const props = defineProps<{
  componentId: string;
  func: MgmtFunction;
  funcRun?: FuncRun;
}>();

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

const statusBoxKind = computed(() => {
  if (managementExecutionStatus.value === "Running") return "loading";
  if (managementExecutionStatus.value === "Failure") return "error";
  if (managementExecutionStatus.value === "Success") return "success";
  return "neutral";
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
