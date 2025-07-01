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
        v-if="
          funcRun &&
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
import FuncRunDetailsLayout from "./layout_components/FuncRunDetailsLayout.vue";
import { assertIsDefined, Context } from "./types";
import { useApi, routes, funcRunTypes } from "./api_composables";
import { keyEmitter } from "./logic_composables/emitters";
import { FuncRun, funcRunStatus } from "./api_composables/func_run";

export interface OutputLine {
  stream: string;
  execution_id: string;
  level: string;
  group?: string;
  message: string;
  timestamp: string;
}

export interface FuncRunLog {
  id: string;
  createdAt: string;
  updatedAt: string;
  funcRunID: string;
  logs: OutputLine[];
  finalized: boolean;
}

const props = defineProps<{
  funcRunId: string;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const router = useRouter();
const isLive = ref(false);

const back = () => {
  const params = router.currentRoute?.value.params ?? {};
  router.push({
    name: "new-hotness",
    params,
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

const api = useApi();
const pollInterval = ref<number | false>(0); // initial calls

const { data: funcRunQuery } = useQuery<Omit<FuncRun, "logs"> | undefined>({
  queryKey: [ctx.changeSetId.value, "funcRun", props.funcRunId],
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
      const timestamp = new Date(log.timestamp).toUTCString();
      return `[${timestamp}] [${log.level.padEnd(5)}] ${log.message}`;
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
