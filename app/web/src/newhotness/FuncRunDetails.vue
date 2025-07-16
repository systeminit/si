<template>
  <FuncRunDetailsLayout
    v-if="funcRun?.id"
    :funcRun="funcRun"
    :status="funcRunStatus(funcRun) || ''"
    :logText="logText"
    :functionCode="functionCode"
    :argsJson="argsJson"
    :resultJson="resultJson"
    :isLive="isLive"
  >
    <template #actions>
      <VButton
        v-if="funcRun && funcRun.componentId && componentExists"
        tone="neutral"
        label="Go to Component"
        size="xs"
        @click="navigateToComponent"
      />
      <VButton
        v-if="
          funcRun &&
          funcRun.actionId &&
          ['Failure', 'ActionFailure', 'Running'].includes(
            funcRunStatus(funcRun) || '',
          )
        "
        tone="destructive"
        label="Remove"
        size="xs"
        @click="removeAction"
      />
      <VButton
        v-if="
          funcRun &&
          funcRun.actionId &&
          ['Failure', 'ActionFailure'].includes(funcRunStatus(funcRun) || '')
        "
        tone="action"
        label="Retry"
        size="xs"
        @click="retryAction"
      />
    </template>
  </FuncRunDetailsLayout>
  <h1 v-else class="text-">Func Run {{ funcRunId }} not found</h1>
</template>

<script lang="ts" setup>
import { computed, onMounted, onBeforeUnmount, ref, inject, unref } from "vue";
import { useQuery } from "@tanstack/vue-query";
import { VButton } from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import { useRouter } from "vue-router";
import { bifrost, useMakeKey, useMakeArgs } from "@/store/realtime/heimdall";
import {
  BifrostComponent,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import FuncRunDetailsLayout from "./layout_components/FuncRunDetailsLayout.vue";
import { assertIsDefined, Context } from "./types";
import { useApi, routes, funcRunTypes } from "./api_composables";
import { keyEmitter } from "./logic_composables/emitters";
import { FuncRun, funcRunStatus, FuncRunLog } from "./api_composables/func_run";
import { preserveExploreState } from "./util";
import { SelectionsInQueryString } from "./Workspace.vue";

const props = defineProps<{
  funcRunId: string;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const router = useRouter();
const isLive = ref(false);

const key = useMakeKey();
const args = useMakeArgs();

const back = () => {
  const params = router.currentRoute?.value.params ?? {};
  router.push({
    name: "new-hotness",
    params,
    query: preserveExploreState(
      router.currentRoute.value?.query as SelectionsInQueryString,
    ),
  });
};

// Action handlers
const removeApi = useApi();
const removeAction = async () => {
  if (funcRun.value?.actionId) {
    const call = removeApi.endpoint(routes.ActionCancel, {
      id: funcRun.value.actionId,
    });

    // This route can mutate head, so we do not need to handle new change set semantics.
    await call.put({});
    router.push({
      name: "new-hotness",
      params: {
        workspacePk: unref(ctx.workspacePk),
        changeSetId: unref(ctx.changeSetId),
      },
    });
  }
};

const retryApi = useApi();
const retryAction = async () => {
  if (funcRun.value?.actionId) {
    const call = retryApi.endpoint(routes.ActionRetry, {
      id: funcRun.value.actionId,
    });

    // This route can mutate head, so we do not need to handle new change set semantics.
    await call.put({});
  }
};

const navigateToComponent = () => {
  if (funcRun.value?.componentId) {
    const params = { ...router.currentRoute.value.params };
    params.componentId = funcRun.value.componentId;
    router.push({
      name: "new-hotness-component",
      params,
      query: preserveExploreState(
        router.currentRoute.value?.query as SelectionsInQueryString,
      ),
    });
  }
};

const api = useApi();
const pollInterval = ref<number | false>(0); // initial calls

const { data: funcRunQuery } = useQuery<Omit<FuncRun, "logs"> | undefined>({
  queryKey: computed(() => [ctx.changeSetId.value, "funcRun", props.funcRunId]),
  queryFn: async () => {
    const call = api.endpoint<funcRunTypes.FuncRunResponse>(routes.FuncRun, {
      id: props.funcRunId,
    });
    const req = await call.get();
    if (api.ok(req)) {
      pollInterval.value = [
        "Running",
        "Dispatched",
        "Created",
        "Failed",
      ].includes(req.data.funcRun.state)
        ? 5000
        : false;
      return req.data.funcRun;
    }
  },
  refetchInterval: () => pollInterval.value,
});

const funcRun = computed(() => funcRunQuery.value);

// Check if the component still exists
const componentId = computed(() => funcRun.value?.componentId);

const { data: componentQuery } = useQuery<BifrostComponent | undefined>({
  queryKey: computed(() => {
    if (!componentId.value) return ["no-component"];
    return key(EntityKind.Component, componentId.value).value;
  }),
  queryFn: async (queryContext) => {
    if (!componentId.value) return undefined;
    return (
      (await bifrost<BifrostComponent>(
        args(EntityKind.Component, componentId.value),
      )) ??
      queryContext.client.getQueryData(
        key(EntityKind.Component, componentId.value).value,
      )
    );
  },
  enabled: computed(() => !!componentId.value),
});

const componentExists = computed(() => !!componentQuery.value);

const { data: funcRunLogsQuery } = useQuery<FuncRunLog | undefined>({
  queryKey: [ctx.changeSetId.value, "funcRunLogs", props.funcRunId],
  queryFn: async () => {
    isLive.value = true;
    const call = api.endpoint<funcRunTypes.FuncRunLogsResponse>(
      routes.FuncRunLogs,
      {
        id: props.funcRunId,
      },
    );
    const req = await call.get();
    if (api.ok(req)) {
      if (req.data.logs.finalized) {
        pollInterval.value = false;
        isLive.value = false;
      }
      return req.data.logs;
    }
  },
  // Automatic polling for running functions every 5 seconds
  refetchInterval: () => pollInterval.value,
  refetchIntervalInBackground: true,
});

const funcRunLogs = computed(() => funcRunLogsQuery.value);

// Format logs as text for CodeViewer
const logText = computed<string>(() => {
  if (!funcRunLogs.value?.logs?.length) return "";

  return funcRunLogs.value.logs
    .map((log) => {
      let timestamp: string;

      // Check if log.timestamp is valid and not empty
      if (!log.timestamp || log.timestamp === "" || log.timestamp === "0") {
        timestamp = "No timestamp";
      } else {
        let date: Date;

        // Handle timestamps that are Unix timestamps in seconds (numbers) vs milliseconds (strings)
        if (typeof log.timestamp === "number") {
          // If it's a number, assume it's Unix timestamp in seconds and convert to milliseconds
          date = new Date(log.timestamp * 1000);
        } else {
          // If it's a string, try parsing as-is first
          date = new Date(log.timestamp);

          // If the parsed date is in 1970 and the original value looks like Unix seconds
          const timeValue = date.getTime();
          if (
            timeValue > 0 &&
            timeValue < 2147483647000 &&
            !log.timestamp.includes("-") &&
            !log.timestamp.includes("T")
          ) {
            // Looks like Unix timestamp in seconds as string, convert it
            const numericTimestamp = parseInt(log.timestamp, 10);
            if (!Number.isNaN(numericTimestamp)) {
              date = new Date(numericTimestamp * 1000);
            }
          }
        }

        const timeValue = date.getTime();
        if (Number.isNaN(timeValue) || timeValue < 0) {
          timestamp = `Invalid: ${log.timestamp}`;
        } else {
          timestamp = date.toUTCString();
        }
      }

      return `[${timestamp}] [${log.level}] ${log.message}`;
    })
    .join("\n");
});

// Format function code for CodeViewer
const functionCode = computed<string>(() => {
  if (!funcRun.value?.functionCodeBase64) return "";

  try {
    const decodedCode = atob(funcRun.value.functionCodeBase64);
    return decodedCode;
  } catch (e) {
    return "// Error decoding function code";
  }
});

// Format arguments for CodeViewer
const argsJson = computed<string>(() => {
  if (!funcRun.value?.functionArgs) return "";

  try {
    return JSON.stringify(funcRun.value.functionArgs, null, 2);
  } catch (e) {
    return "// Error formatting arguments";
  }
});

// Format result for CodeViewer
const resultJson = computed<string>(() => {
  if (!funcRun.value?.resultValue) return "";

  try {
    return JSON.stringify(funcRun.value.resultValue, null, 2);
  } catch (e) {
    return "// Error formatting result";
  }
});

// Set up subscription on component mount
onMounted(() => {
  keyEmitter.on("Escape", () => {
    back();
  });
});

// Ensure cleanup on component unmount
onBeforeUnmount(() => {
  keyEmitter.off("Escape");
});
</script>
