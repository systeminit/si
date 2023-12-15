<template>
  <ScrollArea>
    <template #top>
      <div
        class="flex flex-row gap-xs items-center p-xs m-xs border dark:border-neutral-600 rounded justify-between"
      >
        <div
          class="font-bold text-xl text-center overflow-hidden text-ellipsis flex-grow"
        >
          Test {{ funcStore.selectedFuncDetails?.variant + " " || "" }}Function
          <span class="italic">{{ editingFunc?.name }}</span>
        </div>
        <StatusIndicatorIcon
          v-if="runningTest"
          type="funcTest"
          :status="testStatus"
        />
      </div>

      <div
        class="flex flex-col border dark:border-neutral-600 p-xs m-xs rounded"
      >
        <div class="pb-xs">
          Select the
          <span v-if="assetStore.selectedAsset" class="italic font-bold">
            {{ assetStore.selectedAsset.name }}
          </span>
          component to use as the input for your test:
        </div>
        <div class="flex flex-row items-center gap-sm">
          <VormInput
            v-model="testAttribute"
            class="flex-grow"
            type="dropdown"
            placeholder="no component selected"
            noLabel
            :options="componentAttributeOptions"
            @update:model-value="loadInput"
          />
          <VButton
            label="Run Test"
            size="sm"
            :loading="testStatus === 'running'"
            loadingText="Running"
            loadingIcon="loader"
            icon="play"
            :disabled="!testAttribute || !readyToTest"
            @click="startTest"
          />
        </div>
      </div>
      <!-- DRY RUN SECTION -->
      <div
        v-if="dryRunConfig === 'choose'"
        class="border dark:border-neutral-600 p-xs m-xs rounded"
      >
        <div class="pb-xs">
          Do you want the results of this test to be applied to the component?
        </div>
        <VormInput
          v-model="dryRun"
          class="flex-grow justify-center"
          type="checkbox"
          placeholder="no attribute selected"
          label="Dry Run"
          inlineLabel
          disabled
        />
      </div>
      <div v-else class="py-xs px-sm rounded text-center italic">
        <span
          v-if="dryRunConfig === 'dry'"
          class="text-neutral-500 dark:text-neutral-400"
        >
          The results of this test will not be applied to the component.
        </span>
        <span v-else class="font-bold">
          WARNING: The results of this test will be applied to the component!
        </span>
      </div>
      <!-- END DRY RUN SECTION -->
    </template>

    <TabGroup
      v-if="testAttribute"
      ref="funcTestTabsRef"
      startSelectedTabSlug="input"
      growTabsToFillWidth
    >
      <TabGroupItem label="Input" slug="input">
        <CodeViewer
          :code="testInputCode"
          :title="`Input: ${testComponentDisplayName}`"
        />
      </TabGroupItem>
      <TabGroupItem label="Execution Logs" slug="logs">
        <div class="w-full h-full overflow-hidden flex flex-col absolute">
          <template v-if="rawTestLogs.length > 0">
            <!-- TODO(WENDY) - a chip here to show output info -->
            <div
              class="border dark:border-neutral-600 dark:bg-shade-100 bg-neutral-100 rounded-xl m-xs p-xs flex flex-row items-center gap-xs flex-none"
            >
              <StatusIndicatorIcon
                :status="testLogs.status"
                type="funcTest"
                size="2xl"
              />
              <div class="text-xl font-bold capitalize">
                Status: {{ testLogs.status }}
              </div>
              <div class="flex-grow text-right">
                <a
                  class="text-action-400 font-bold text-sm hover:underline cursor-pointer"
                  @click="additionalOutputInfoModalRef.open"
                >
                  Additional Output Info
                </a>
                <Modal
                  ref="additionalOutputInfoModalRef"
                  :title="`Output Information For Test On ${testComponentDisplayName}`"
                >
                  <div class="w-full max-h-[50vh] relative overflow-auto">
                    <CodeViewer
                      :code="testLogs.output"
                      :title="`Output Info: ${testComponentDisplayName}`"
                    />
                  </div>
                </Modal>
              </div>
            </div>

            <div
              v-if="testLogs.stdout"
              class="relative flex-shrink overflow-auto basis-full"
            >
              <CodeViewer
                :code="testLogs.stdout"
                :title="`stdout: ${testComponentDisplayName}`"
                showTitle
              />
            </div>
            <div
              v-else
              class="border dark:border-neutral-600 rounded p-xs m-sm text-center text-neutral-500 dark:text-neutral-400 italic"
            >
              No stdout logs to show.
            </div>
            <div
              v-if="testLogs.stderr"
              class="relative flex-shrink overflow-auto basis-full"
            >
              <CodeViewer
                :code="testLogs.stderr"
                :title="`stderr: ${testComponentDisplayName}`"
                showTitle
              />
            </div>
            <div
              v-else
              class="border dark:border-neutral-600 rounded p-xs m-sm text-center text-neutral-500 dark:text-neutral-400 italic"
            >
              No stderr logs to show.
            </div>
          </template>
          <div
            v-else-if="runningTest"
            class="w-full p-md text-center text-neutral-500 dark:text-neutral-400 flex flex-col items-center"
          >
            <template v-if="testStatus === 'running'">
              <div class="pb-sm">
                Awaiting logs for the currently running test...
              </div>
              <StatusIndicatorIcon
                type="funcTest"
                :status="testStatus"
                size="2xl"
                tone="neutral"
              />
            </template>
            <template v-else>No logs available for this test.</template>
          </div>
          <div
            v-else
            class="w-full p-md text-center text-neutral-500 dark:text-neutral-400"
          >
            Run a test to see the execution logs.
          </div>
        </div>
      </TabGroupItem>
      <TabGroupItem label="Output" slug="output">
        <CodeViewer
          v-if="testOutputCode"
          :code="testOutputCode"
          :title="`Output: ${testComponentDisplayName}`"
        />
        <div
          v-else-if="runningTest"
          class="w-full p-md text-center text-neutral-500 dark:text-neutral-400 flex flex-col items-center"
        >
          <template v-if="testStatus === 'running'">
            <div class="pb-sm">
              Awaiting output for the currently running test...
            </div>
            <StatusIndicatorIcon
              type="funcTest"
              :status="testStatus"
              size="2xl"
              tone="neutral"
            />
          </template>
          <template v-else>No output available for this test.</template>
        </div>
        <div
          v-else
          class="w-full p-md text-center text-neutral-500 dark:text-neutral-400"
        >
          Run a test to see the output.
        </div>
      </TabGroupItem>
    </TabGroup>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  VButton,
  VormInput,
  ScrollArea,
  TabGroupItem,
  TabGroup,
  Modal,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { useComponentsStore } from "@/store/components.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { FuncVariant } from "@/api/sdf/dal/func";
import CodeViewer from "../CodeViewer.vue";
import StatusIndicatorIcon, { Status } from "../StatusIndicatorIcon.vue";

const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const assetStore = useAssetStore();
const realtimeStore = useRealtimeStore();
const changeSetStore = useChangeSetsStore();

const asset = computed(() => assetStore.selectedAsset);

const storeFuncDetails = computed(() => funcStore.selectedFuncDetails);
const editingFunc = ref(_.cloneDeep(storeFuncDetails.value));

const additionalOutputInfoModalRef = ref();

const funcTestTabsRef = ref();
const testAttribute = ref(undefined);
const dryRun = ref(true);
const testInputCode = ref("");
const testInputProperties = ref<Record<string, unknown> | null>();
const testOutputCode = ref("");
const readyToTest = ref(false);
const runningTest = ref(false);

const dryRunConfig = computed(() => {
  // TODO(Wendy) - which function variants allow for a choice of dry run? which are always dry and which are always wet?
  // Note(Paulo): We only support dry run when testing functions
  if (funcStore.selectedFuncDetails?.variant === FuncVariant.Attribute) {
    return "dry";
    // eslint-disable-next-line no-constant-condition
  } else if (false) {
    return "choose";
  } else {
    // return "wet";
    return "dry";
  }
});

const components = computed(() => {
  return componentsStore.allComponents.filter(
    (c) => c.schemaVariantId === asset.value?.schemaVariantId,
  );
});

const componentAttributeOptions = computed(() => {
  return components.value.map((c) => {
    return { value: c.id, label: c.displayName };
  });
});

const testComponentDisplayName = computed(() => {
  if (testAttribute.value) {
    return componentsStore.componentsById[testAttribute.value]?.displayName;
  } else return "ERROR";
});

const testStatus = computed(() => {
  const status = funcStore.getRequestStatus("EXECUTE").value;

  if (status.isPending) return "running";
  else if (status.isSuccess) return "success";
  else return "failure";
});
const rawTestLogs = ref<
  { stream: string; level: string; message: string; timestamp: string }[]
>([]);
const testLogs = computed(() => {
  const logs = {
    stdout: "",
    stderr: "",
    output: "",
    status: "running" as Status,
  };
  if (rawTestLogs.value && rawTestLogs.value.length > 0) {
    rawTestLogs.value.forEach((log) => {
      if (log.stream === "stdout") {
        if (logs.stdout !== "") logs.stdout += "\n";
        logs.stdout += log.message;
      } else if (log.stream === "stderr") {
        if (logs.stderr !== "") logs.stderr += "\n";
        logs.stderr += log.message;
      } else if (
        log.stream === "output" &&
        log.message.slice(0, 8) === "Output: "
      ) {
        logs.output = log.message.slice(8);
        const outputJSON = JSON.parse(logs.output);
        logs.status = (outputJSON.status as Status) ?? "unknown";
      }
    });
  }

  return logs;
});

const resetTestData = () => {
  testInputCode.value = "";
  testOutputCode.value = "";
  rawTestLogs.value = [];
  readyToTest.value = false;
  runningTest.value = false;
};

const loadInput = async () => {
  await prepareTest();
  if (funcTestTabsRef.value && funcTestTabsRef.value.tabExists("input")) {
    funcTestTabsRef.value.selectTab("input");
  }
};

const prepareTest = async () => {
  if (!funcStore.selectedFuncId || !testAttribute.value) return;

  resetTestData();

  const res = await componentsStore.FETCH_COMPONENT_JSON(testAttribute.value);
  if (!res.result.success) {
    // TODO(Wendy) - handle a failure properly instead of just bailing!
    return;
  }

  const json = res.result.data.json;
  const selectedFunc = funcStore.selectedFuncDetails;
  if (selectedFunc?.associations?.type === "attribute") {
    const prototypes = selectedFunc.associations.prototypes;

    const getJsonPath = () => {
      for (const prototype of prototypes) {
        for (const arg of prototype.prototypeArguments) {
          const prop = funcStore.propForInternalProviderId(
            arg.internalProviderId ?? "",
          );

          if (prop) {
            return `${prop.path}${prop.name}`;
          }
        }
      }
    };
    const jsonPath = getJsonPath();
    if (!jsonPath) {
      // TODO(Wendy) - handle a failure properly instead of just bailing!
      return;
    }
    // We remove the first two strings because they will always be an empty string and "root"
    const jsonPathArray = jsonPath.split("/").splice(2);
    const props: Record<string, unknown> | null = json as Record<
      string,
      unknown
    >;

    let properties: Record<string, unknown> | null = {};
    for (const key of jsonPathArray) {
      if (!properties[key]) {
        properties = null;
        break;
      }
      properties = properties[key] as Record<string, unknown>;
    }
    if (jsonPathArray[jsonPathArray.length - 1]) {
      const last = jsonPathArray[jsonPathArray.length - 1] as string;
      properties = { [last]: properties };
    }

    testInputCode.value = JSON.stringify(properties, null, 2);
    testInputProperties.value = properties;
  } else if (selectedFunc?.associations?.type === "action") {
    const properties: Record<string, unknown> | null = json as Record<
      string,
      unknown
    >;

    testInputCode.value = JSON.stringify(properties, null, 2);
    testInputProperties.value = properties;
  } else if (selectedFunc?.associations?.type === "validation") {
    const prototypes = selectedFunc.associations.prototypes;

    const getJsonPath = () => {
      for (const prototype of prototypes) {
        const prop = funcStore.propForId(prototype.propId);

        if (prop) {
          return `${prop.path}${prop.name}`;
        }
      }
    };
    const jsonPath = getJsonPath();
    if (!jsonPath) {
      // TODO(Wendy) - handle a failure properly instead of just bailing!
      return;
    }
    // We remove the first two strings because they will always be an empty string and "root"
    const jsonPathArray = jsonPath.split("/").splice(2);
    let properties: Record<string, unknown> | null = json as Record<
      string,
      unknown
    >;

    for (const key of jsonPathArray) {
      if (!properties[key]) {
        properties = null;
        break;
      }
      properties = properties[key] as Record<string, unknown>;
    }

    testInputCode.value = JSON.stringify(properties, null, 2);
    testInputProperties.value = properties;
  } else if (
    selectedFunc?.associations?.type === "codeGeneration" ||
    selectedFunc?.associations?.type === "qualification"
  ) {
    const props: Record<string, unknown> | null = json as Record<
      string,
      unknown
    >;

    const toSnakeCase = (inputString: string) => {
      return inputString
        .split("")
        .map((character) => {
          if (character === character.toUpperCase()) {
            return `_${character.toLowerCase()}`;
          } else {
            return character;
          }
        })
        .join("");
    };

    const properties: Record<string, unknown> = {};
    for (const input of selectedFunc.associations.inputs) {
      if (!props) break;

      const key = toSnakeCase(`${input}`);
      properties[key] = props[key];
    }

    testInputCode.value = JSON.stringify(properties, null, 2);
    testInputProperties.value = properties;
  } else {
    // TODO(Wendy) - handle a failure properly instead of just bailing!
    return;
  }

  readyToTest.value = true;
};

const startTest = async () => {
  if (!funcStore.selectedFuncId || !testAttribute.value || !readyToTest.value)
    return;

  prepareTest();
  readyToTest.value = false;

  const executionKey = new Date().toString() + _.random();

  const selectedChangeSetId = changeSetStore.selectedChangeSet?.id;

  realtimeStore.subscribe(executionKey, `changeset/${selectedChangeSetId}`, [
    {
      eventType: "LogLine",
      callback: (logLine) => {
        if (logLine.executionKey === executionKey) {
          rawTestLogs.value.push(logLine.stream);
        }
      },
    },
  ]);

  // Run the test!
  runningTest.value = true;
  rawTestLogs.value = [];
  funcTestTabsRef.value.selectTab("logs");

  let args = testInputProperties.value;
  if (funcStore.selectedFuncDetails?.associations?.type === "validation") {
    args = { value: args };
  } else if (funcStore.selectedFuncDetails?.associations?.type === "action") {
    args = { kind: "standard", properties: args };
  }

  const output = await funcStore.EXECUTE({
    id: funcStore.selectedFuncId,
    args,
    executionKey,
    componentId: testAttribute.value,
  });

  realtimeStore.unsubscribe(executionKey);
  readyToTest.value = true;

  if (output.result.success) {
    testOutputCode.value = JSON.stringify(output.result.data.output, null, 2);
    rawTestLogs.value = output.result.data.logs;
  } else {
    testOutputCode.value = "ERROR: Test Failed To Run";
  }
};
</script>
