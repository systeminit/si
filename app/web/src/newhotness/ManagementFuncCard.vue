<template>
  <li
    v-if="func.kind !== 'import'"
    class="rounded border border-neutral-600 flex flex-col"
  >
    <div class="border-b border-neutral-600 w-full p-xs">
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
        :loading="dispatchedFunc || funcRun?.state === 'Running'"
        @click="runMgmtFunc(func.id)"
      />
    </div>
    <div
      v-if="funcRun"
      class="m-sm bg-neutral-600 flex items-center gap-xs px-xs py-sm rounded"
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
import { Icon, IconNames, VButton } from "@si/vue-lib/design-system";
import { useRoute, useRouter } from "vue-router";
import { inject, ref } from "vue";
import { routes, useApi } from "@/newhotness/api_composables";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import { ExploreContext } from "@/newhotness/types";
import { MgmtFunction } from "@/workers/types/entity_kind_types";

const props = defineProps<{
  componentId: string;
  func: MgmtFunction;
  funcRun?: FuncRun;
}>();

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");

const router = useRouter();
const route = useRoute();

const mgmtRunApi = useApi();
const runMgmtFunc = async (funcId: string) => {
  const call = mgmtRunApi.endpoint<{ success: boolean }>(
    routes.RunMgmtPrototype,
    {
      prototypeId: funcId,
      componentId: props.componentId,
      viewId: explore?.viewId.value ?? "DEFAULT", // Should get the default view id
    },
  );

  const { req, newChangeSetId } = await call.post({});

  dispatchedFunc.value = true;
  setTimeout(() => {
    dispatchedFunc.value = false;
  }, 2000);

  // NOTE(nick): need to make sure this makes sense after the timeout.
  if (mgmtRunApi.ok(req) && newChangeSetId) {
    mgmtRunApi.navigateToNewChangeSet(
      {
        name: "new-hotness",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
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
