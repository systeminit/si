<template>
  <ScrollArea>
    <template #top>
      <div
        class="flex flex-row gap-xs items-center p-xs justify-between border-y border-neutral-200 dark:border-neutral-600"
      >
        <div class="font-bold text-xl text-center overflow-hidden text-ellipsis flex-grow break-words">
          Test {{ funcStore.selectedFuncSummary?.kind + " " || "" }}Function
          <span class="italic"
            >"{{ funcStore.selectedFuncSummary?.displayName || funcStore.selectedFuncSummary?.name }}"</span
          >
        </div>
        <StatusIndicatorIcon v-if="runningTest" :status="testStatus" type="funcTest" />
      </div>

      <div class="flex flex-col border dark:border-neutral-600 p-xs m-xs rounded">
        <div class="pb-xs">
          Select the
          <span v-if="assetStore.selectedSchemaVariant" class="italic font-bold">
            {{ assetStore.selectedSchemaVariant.schemaName }}
          </span>
          component to use as the input for your test:
        </div>
        <FuncTestSelector
          v-if="selectedAsset"
          ref="funcTestSelectorRef"
          :isAttributeFunc="funcStore.selectedFuncSummary?.kind === FuncKind.Attribute"
          :readyToTest="readyToTest"
          :schemaVariantId="selectedAsset?.schemaVariantId"
          :testStatus="testStatus"
          @loadInput="loadInput"
          @startTest="startTest"
        />
      </div>
      <!-- DRY RUN SECTION -->
      <div v-if="dryRunConfig === 'choose'" class="border dark:border-neutral-600 p-xs m-xs rounded">
        <div class="pb-xs">Do you want the results of this test to be applied to the component?</div>
        <VormInput
          v-model="dryRun"
          class="flex-grow justify-center"
          disabled
          inlineLabel
          label="Dry Run"
          placeholder="no attribute selected"
          type="checkbox"
        />
      </div>
      <div v-else class="py-xs px-sm rounded text-center italic">
        <span v-if="dryRunConfig === 'dry'" class="text-neutral-500 dark:text-neutral-400">
          The results of this test will not be applied to the component.
        </span>
        <span v-else class="font-bold"> WARNING: The results of this test will be applied to the component! </span>
      </div>
      <!-- END DRY RUN SECTION -->
    </template>

    <TabGroup
      v-if="enableTestTabGroup"
      ref="funcTestTabsRef"
      growTabsToFillWidth
      marginTop="2xs"
      startSelectedTabSlug="input"
      variant="secondary"
    >
      <TabGroupItem label="Input" slug="input">
        <CodeViewer :code="testInputCode" :title="`Input: ${testComponentDisplayName}`" />
      </TabGroupItem>
      <TabGroupItem label="Execution Logs" slug="logs">
        <div class="w-full h-full overflow-hidden flex flex-col absolute">
          <template v-if="rawTestLogs.length > 0">
            <!-- TODO(WENDY) - a chip here to show output info -->
            <div
              class="border dark:border-neutral-600 dark:bg-shade-100 bg-neutral-100 rounded-xl m-xs p-xs flex flex-row items-center gap-xs flex-none"
            >
              <StatusIndicatorIcon :status="testLogs.status" size="2xl" type="funcTest" />
              <div class="text-xl font-bold capitalize">Status: {{ testLogs.status }}</div>
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
                    <CodeViewer :code="testLogs.output" :title="`Output Info: ${testComponentDisplayName}`" />
                  </div>
                </Modal>
              </div>
            </div>

            <div v-if="testLogs.stdout" class="relative flex-shrink overflow-auto basis-full">
              <CodeViewer :code="testLogs.stdout" :title="`stdout: ${testComponentDisplayName}`" showTitle />
            </div>
            <div
              v-else
              class="border dark:border-neutral-600 rounded p-xs m-sm text-center text-neutral-500 dark:text-neutral-400 italic"
            >
              No stdout logs to show.
            </div>
            <div v-if="testLogs.stderr" class="relative flex-shrink overflow-auto basis-full">
              <CodeViewer :code="testLogs.stderr" :title="`stderr: ${testComponentDisplayName}`" showTitle />
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
              <div class="pb-sm">Awaiting logs for the currently running test...</div>
              <StatusIndicatorIcon :status="testStatus" size="2xl" tone="neutral" type="funcTest" />
            </template>
            <template v-else>No logs available for this test.</template>
          </div>
          <div v-else class="w-full p-md text-center text-neutral-500 dark:text-neutral-400">
            Run a test to see the execution logs.
          </div>
        </div>
      </TabGroupItem>
      <TabGroupItem label="Output" slug="output">
        <CodeViewer v-if="testOutputCode" :code="testOutputCode" :title="`Output: ${testComponentDisplayName}`" />
        <div
          v-else-if="runningTest"
          class="w-full p-md text-center text-neutral-500 dark:text-neutral-400 flex flex-col items-center"
        >
          <template v-if="testStatus === 'running'">
            <div class="pb-sm">Awaiting output for the currently running test...</div>
            <StatusIndicatorIcon :status="testStatus" size="2xl" tone="neutral" type="funcTest" />
          </template>
          <template v-else>No output available for this test.</template>
        </div>
        <div v-else class="w-full p-md text-center text-neutral-500 dark:text-neutral-400">
          Run a test to see the output.
        </div>
      </TabGroupItem>
    </TabGroup>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { VormInput, ScrollArea, TabGroupItem, TabGroup, Modal } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { useComponentsStore } from "@/store/components.store";
import { FuncKind, LeafInputLocation } from "@/api/sdf/dal/func";
import { useFuncRunsStore } from "@/store/func_runs.store";
import FuncTestSelector from "./FuncTestSelector.vue";
import CodeViewer from "../CodeViewer.vue";
import StatusIndicatorIcon, { Status } from "../StatusIndicatorIcon.vue";

export type TestStatus = "running" | "success" | "failure";

const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const assetStore = useAssetStore();
const funcRunsStore = useFuncRunsStore();

const additionalOutputInfoModalRef = ref();
const funcTestSelectorRef = ref<InstanceType<typeof FuncTestSelector>>();

const selectedAsset = computed(() => assetStore.selectedSchemaVariant);

const enableTestTabGroup = computed((): boolean => {
  if (funcStore.selectedFuncSummary?.kind === FuncKind.Attribute) {
    if (funcTestSelectorRef.value?.selectedComponentId && funcTestSelectorRef.value?.selectedOutputLocationId) {
      return true;
    }
    return false;
  }
  if (funcTestSelectorRef.value?.selectedComponentId) {
    return true;
  }
  return false;
});

const funcTestTabsRef = ref();
const dryRun = ref(true);
const testInputCode = ref("");
const testInputProperties = ref<Record<string, unknown> | null>();
const testOutputCode = ref("");
const testOutput = ref<unknown>(null);
const readyToTest = ref(false);
const runningTest = ref(false);

const dryRunConfig = computed(() => {
  // TODO(Wendy) - which function variants allow for a choice of dry run? which are always dry and which are always wet?
  // Note(Paulo): We only support dry run when testing functions
  if (funcStore.selectedFuncSummary?.kind === FuncKind.Attribute) {
    return "dry";
  } else {
    // return "wet";
    return "dry";
  }
});

const testComponentDisplayName = computed(() => {
  if (funcTestSelectorRef.value?.selectedComponentId) {
    return componentsStore.allComponentsById[funcTestSelectorRef.value.selectedComponentId]?.def.displayName;
  } else return "ERROR";
});

const testStatus = computed((): TestStatus => {
  const status = funcStore.getRequestStatus("TEST_EXECUTE").value;

  if (status.isPending) return "running";
  else if (status.isSuccess) return "success";
  else return "failure";
});
const rawTestLogs = ref<{ stream: string; level: string; message: string; timestamp: string }[]>([]);
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
      } else if (log.stream === "output" && log.message.slice(0, 8) === "Output: ") {
        logs.output = log.message.slice(8);
        const outputJSON = JSON.parse(logs.output);
        logs.status = (outputJSON.status as Status) ?? "unknown";
      }
    });
    if (logs.status === "running" && testOutput.value) {
      logs.status = (testOutput.value as { result: Status }).result ?? "running";
    }
  }

  return logs;
});

const resetTestData = () => {
  testInputCode.value = "";
  testOutputCode.value = "";
  testOutput.value = null;
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
  if (!funcStore.selectedFuncId || !funcTestSelectorRef.value?.selectedComponentId) return;

  resetTestData();

  if (funcStore.selectedFuncSummary?.kind === FuncKind.Attribute) {
    if (!funcTestSelectorRef.value.selectedOutputLocationId) {
      throw new Error("cannot prepare test for attribute func without a selected output location");
    }

    const propId = funcTestSelectorRef.value.selectedOutputLocationId.startsWith("p_")
      ? funcTestSelectorRef.value.selectedOutputLocationId.replace("p_", "")
      : undefined;
    const outputSocketId = funcTestSelectorRef.value.selectedOutputLocationId.startsWith("s_")
      ? funcTestSelectorRef.value.selectedOutputLocationId.replace("s_", "")
      : undefined;

    const res = await funcStore.FETCH_PROTOTYPE_ARGUMENTS(propId, outputSocketId);
    if (!res.result.success) {
      throw new Error("could not fetch prototype arguments needed for preparing test");
    }

    const preparedArguments = res.result.data.preparedArguments;

    let properties: Record<string, unknown> = {};
    properties = preparedArguments;
    testInputCode.value = JSON.stringify(properties, null, 2);
    testInputProperties.value = properties;
  } else if (
    funcStore.selectedFuncSummary?.kind === FuncKind.CodeGeneration ||
    funcStore.selectedFuncSummary?.kind === FuncKind.Qualification
  ) {
    const res = await componentsStore.FETCH_COMPONENT_JSON(funcTestSelectorRef.value.selectedComponentId);
    if (!res.result.success) {
      throw new Error("could not fetch component json needed for preparing test");
    }

    const json = res.result.data.json;

    const props: Record<string, unknown> | null = json as Record<string, unknown>;

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
    let selectedInputs = [] as LeafInputLocation[];

    if (funcStore.selectedFuncSummary?.kind === FuncKind.CodeGeneration)
      selectedInputs =
        funcStore.codegenBindings[funcStore.selectedFuncId]?.find((b) => {
          const schemaVariantId =
            componentsStore.allComponentsById[funcTestSelectorRef.value?.selectedComponentId || ""]?.def
              .schemaVariantId;
          if (schemaVariantId) return b.schemaVariantId === schemaVariantId;
          return false;
        })?.inputs || [];

    if (funcStore.selectedFuncSummary?.kind === FuncKind.Qualification)
      selectedInputs =
        funcStore.qualificationBindings[funcStore.selectedFuncId]?.find((b) => {
          const schemaVariantId =
            componentsStore.allComponentsById[funcTestSelectorRef.value?.selectedComponentId || ""]?.def
              .schemaVariantId;
          if (schemaVariantId) return b.schemaVariantId === schemaVariantId;
          return false;
        })?.inputs || [];

    if (props)
      for (const input of selectedInputs) {
        const key = toSnakeCase(`${input}`);
        properties[key] = props[key];
      }

    testInputCode.value = JSON.stringify(properties, null, 2);
    testInputProperties.value = properties;
  } else if (funcStore.selectedFuncSummary?.kind === FuncKind.Management) {
    const res = await componentsStore.FETCH_COMPONENT_JSON(funcTestSelectorRef.value.selectedComponentId);
    if (!res.result.success) {
      throw new Error("could not fetch component json needed for preparing test");
    }

    const json = res.result.data.json;
    const props: Record<string, unknown> | null = json as Record<string, unknown>;

    const geometry = { x: 0, y: 0, width: 500, height: 500 };
    // TODO recursive toSnakeCase all props
    const thisComponent = {
      this_component: {
        properties: props,
        geometry,
      },
    };
    testInputCode.value = JSON.stringify(thisComponent, null, 2);
    testInputProperties.value = thisComponent;
  } else {
    // This should not be possible since we should only prepare tests for valid func kinds.
    return;
  }

  readyToTest.value = true;
};

const startTest = async () => {
  if (!funcStore.selectedFuncCode || !funcTestSelectorRef.value?.selectedComponentId || !readyToTest.value) return;

  prepareTest();
  readyToTest.value = false;

  // Run the test!
  runningTest.value = true;
  rawTestLogs.value = [];
  funcTestTabsRef.value.selectTab("logs");

  const args = testInputProperties.value;

  const response = await funcStore.TEST_EXECUTE({
    funcId: funcStore.selectedFuncCode.funcId,
    args,
    code: funcStore.selectedFuncCode.code,
    componentId: funcTestSelectorRef.value?.selectedComponentId,
  });

  readyToTest.value = true;

  // TODO: @brit - currently test does not create a changeset
  // so this will work fine
  // but once we force a change set, this will need to change
  if (response.result.success) {
    const funcRunId = response.result.data.funcRunId;
    await funcRunsStore.GET_FUNC_RUN(funcRunId);
    const funcRun = funcRunsStore.funcRuns[funcRunId];

    if (funcRun) {
      testOutput.value = funcRun.resultValue;
      testOutputCode.value = JSON.stringify(funcRun.resultValue, null, 2);
      if (funcRun.logs) {
        rawTestLogs.value = funcRun.logs.logs;
      }
    } else {
      testOutput.value = null;
      testOutputCode.value = "ERROR: Test Execution Request Succeeded, But Func Run Not Found";
    }
  } else {
    testOutput.value = null;
    testOutputCode.value = "ERROR: Test Failed To Run";
  }
};
</script>
