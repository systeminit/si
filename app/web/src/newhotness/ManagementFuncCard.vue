<template>
  <li
    v-if="func.kind !== MgmtFuncKind.Import"
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
          'border-b w-full p-xs',
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
        v-if="func.kind !== MgmtFuncKind.RunTemplate"
        size="sm"
        label="Run Function"
        iconTone="action"
        loadingText="Running"
        :loading="dispatchedFunc || funcRun?.state === 'Running'"
        @click="runMgmtFunc(func.id)"
      />
    </div>
    <div
      v-if="funcRun"
      :class="
        clsx(
          'm-sm flex items-center gap-xs px-xs py-sm rounded',
          themeClasses('bg-neutral-300', 'bg-neutral-700'),
        )
      "
    >
      <Icon :name="getIconNameFromFuncRun(funcRun)" class="shrink-0" />
      <span class="grow">
        {{ getMessageFromFuncRun(funcRun) }}
      </span>
      <VButton
        size="sm"
        label="See Func Run"
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
import { ref } from "vue";
import clsx from "clsx";
import { routes, useApi } from "@/newhotness/api_composables";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import { MgmtFuncKind, MgmtFunction } from "@/workers/types/entity_kind_types";

const props = defineProps<{
  componentId: string;
  func: MgmtFunction;
  funcRun?: FuncRun;
}>();

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

const getIconNameFromFuncRun = (funcRun: FuncRun): IconNames => {
  if (funcRun.state === "Success") return "check-circle";
  if (funcRun.state === "Failure") return "x-circle";

  return "loader";
};

const getMessageFromFuncRun = (funcRun: FuncRun): string => {
  if (funcRun.state === "Success")
    return "Executed successfully. Click to see execution results";
  if (funcRun.state === "Failure")
    return `Error executing function, click "See Func Run" to get more details`;

  return "Running Function...";
};

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
