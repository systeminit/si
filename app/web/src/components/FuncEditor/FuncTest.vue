<template>
  <ScrollArea>
    <template #top>
      <div
        class="flex flex-row gap-xs items-center p-xs m-xs border dark:border-neutral-600 rounded"
      >
        <div
          class="text-xl font-bold text-center flex-grow overflow-hidden text-ellipsis"
        >
          Test Attribute Function
          <span class="italic">{{ editingFunc?.name }}</span>
        </div>

        <template v-if="testStarted">
          <VButton label="New Test" size="sm" tone="success" @click="newTest" />
          <VButton
            label="Rerun"
            size="sm"
            :disabled="testStatus === 'running'"
            @click="startTest"
          />
          <StatusIndicatorIcon type="funcTest" :status="testStatus" />
        </template>
      </div>
    </template>
    <template v-if="testStarted">
      <TabGroup startSelectedTabSlug="logs" growTabsToFillWidth>
        <TabGroupItem label="Input" slug="input">
          <CodeViewer
            :code="testInputCode"
            :title="`Input: ${testComponentDisplayName}`"
          />
        </TabGroupItem>
        <TabGroupItem label="Execution Logs" slug="logs">
          <ScrollArea>
            <CodeViewer
              v-for="(log, index) in testLogs"
              :key="index"
              :code="log"
              :title="`Log: ${testComponentDisplayName}`"
            />
          </ScrollArea>
        </TabGroupItem>
        <TabGroupItem label="Output" slug="output">
          <CodeViewer
            :code="testOutputCode"
            :title="`Output: ${testComponentDisplayName}`"
          />
        </TabGroupItem>
      </TabGroup>
    </template>
    <template v-else>
      <div class="border dark:border-neutral-600 p-xs m-xs rounded">
        <div class="pb-xs">
          Select the bound component attribute to use as the input for your
          test:
        </div>
        <div class="flex flex-row items-center gap-sm">
          <VormInput
            v-model="testAttribute"
            class="flex-grow"
            type="dropdown"
            placeholder="no attribute selected"
            noLabel
            :options="componentAttributeOptions"
          />
          <VButton label="Create New" />
        </div>
      </div>
      <div class="border dark:border-neutral-600 p-xs m-xs rounded">
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
        <!-- TODO(Wendy) - currently testing is dry run only, need to implement not dry running -->
      </div>
      <div class="pt-sm m-xs flex flex-row gap-sm items-center justify-center">
        <VButton
          label="Start"
          tone="action"
          size="lg"
          :disabled="!testAttribute"
          @click="startTest"
        />
      </div>
    </template>
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
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { useComponentsStore } from "@/store/components.store";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import CodeViewer from "../CodeViewer.vue";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";

const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const assetStore = useAssetStore();
const realtimeStore = useRealtimeStore();
const changeSetStore = useChangeSetsStore();

const asset = computed(() => assetStore.selectedAsset);

const storeFuncDetails = computed(() => funcStore.selectedFuncDetails);
const editingFunc = ref(_.cloneDeep(storeFuncDetails.value));

const testAttribute = ref(undefined);
const dryRun = ref(true);
const testInputCode = ref("");
const testOutputCode = ref("");

const components = computed(() => {
  return componentsStore.allComponents.filter(
    (c) => c.schemaVariantId === asset.value?.schemaVariantId,
  );
});

const componentAttributeOptions = computed(() => {
  return components.value.map((c) => {
    // TODO(Wendy) - make the label a bit clearer!
    return { value: c.id, label: c.displayName };
  });
});

const testComponentDisplayName = computed(() => {
  if (testAttribute.value) {
    return componentsStore.componentsById[testAttribute.value]?.displayName;
  } else return "ERROR";
});

const testStarted = ref(false); // TODO(Wendy) - we should make this persist!
const testStatus = computed(() => {
  const status = funcStore.getRequestStatus("EXECUTE").value;

  if (status.isPending) return "running";
  else if (status.isSuccess) return "success";
  else return "failure";
});
const testLogs = ref<string[]>([]);

const resetTestData = () => {
  testInputCode.value = "";
  testOutputCode.value = "";
  testLogs.value = [];
};

const startTest = async () => {
  if (!funcStore.selectedFuncId || !testAttribute.value) return;

  resetTestData();

  const res = await componentsStore.FETCH_COMPONENT_JSON(testAttribute.value);
  if (!res.result.success) {
    // TODO(Wendy) - handle a failure properly instead of just bailing!
    return;
  }

  const json = res.result.data.json;
  const selectedFunc = funcStore.selectedFuncDetails;
  if (selectedFunc?.associations?.type !== "attribute") {
    // TODO(Wendy) - handle a failure properly instead of just bailing!
    return;
  }
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

  testInputCode.value = JSON.stringify(properties);
  testStarted.value = true;

  const executionKey = new Date().toString() + _.random();

  const selectedChangeSetId = changeSetStore.selectedChangeSetId;

  realtimeStore.subscribe(executionKey, `changeset/${selectedChangeSetId}`, [
    {
      eventType: "LogLine",
      callback: (logLine) => {
        if (logLine.executionKey === executionKey) {
          testLogs.value.push(logLine.stream.message);
        }
      },
    },
  ]);

  const output = await funcStore.EXECUTE({
    id: funcStore.selectedFuncId,
    args: { properties },
    executionKey,
  });

  realtimeStore.unsubscribe(executionKey);

  if (output.result.success) {
    testOutputCode.value = JSON.stringify(output.result.data.output);
    testLogs.value = output.result.data.logs.map((log) => {
      return log.message;
    });
  } else {
    testOutputCode.value = "ERROR: Test Failed To Run";
  }
};

const newTest = () => {
  testStarted.value = false;
};
</script>
