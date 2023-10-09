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
          <VButton label="Rerun" size="sm" />
          <StatusIndicatorIcon type="funcTest" status="running" />
        </template>
      </div>
    </template>
    <template v-if="testStarted">
      <TabGroup startSelectedTabSlug="logs" growTabsToFillWidth>
        <TabGroupItem label="Input" slug="input">
          <CodeViewer
            :code="`this is a mock\n\n\ntrans rights`"
            :title="`Input: ${testAttribute}`"
          />
        </TabGroupItem>
        <TabGroupItem label="Execution Logs" slug="logs">
          <ScrollArea>
            <div
              class="border dark:border-neutral-600 p-xs m-xs rounded text-center"
            >
              event 1
            </div>
            <div
              class="border dark:border-neutral-600 p-xs m-xs rounded text-center"
            >
              event 2
            </div>
            <div
              class="border dark:border-neutral-600 p-xs m-xs rounded text-center"
            >
              event 3
            </div>
          </ScrollArea>
        </TabGroupItem>
        <TabGroupItem label="Output" slug="output">
          <CodeViewer
            code="testing test test mock time"
            :title="`Output: ${testAttribute}`"
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
        />
      </div>
      <div class="pt-sm m-xs flex flex-row gap-sm items-center justify-center">
        <VButton label="Start" tone="action" size="lg" @click="startTest" />
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
import CodeViewer from "../CodeViewer.vue";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";

const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const assetStore = useAssetStore();

const asset = computed(() => assetStore.selectedAsset);

const storeFuncDetails = computed(() => funcStore.selectedFuncDetails);
const editingFunc = ref(_.cloneDeep(storeFuncDetails.value));

const testAttribute = ref(undefined);
const dryRun = ref(false);

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

const testStarted = ref(false); // TODO(Wendy) - we should make this persist!

const startTest = () => {
  testStarted.value = true;
};

const newTest = () => {
  testStarted.value = false;
};
</script>
