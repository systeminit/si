<template>
  <div v-if="viewStore.selectedComponent" class="flex flex-col h-full w-full">
    <div
      v-if="!props.component.def.resourceId"
      class="text-xs text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
    >
      These functions can require the resource identifier, enter it here
    </div>

    <div class="ml-xs mt-xs">
      <VormInput
        v-model="resourceId"
        compact
        type="text"
        label="Resource Id"
        @blur="saveResource"
      />
    </div>

    <span class="uppercase font-bold p-xs mt-sm">FUNCTION LIST</span>
    <div
      class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
    >
      The functions below will run immediately in a change set
    </div>
    <ul class="text-sm">
      <template
        v-for="prototype in funcStore.managementFunctionsForSelectedComponent"
        :key="prototype.managementPrototypeId"
      >
        <ManagementRunPrototype
          :prototype="prototype"
          :component="props.component"
          @showLatestRunTab="showLatestRunTab"
          @runUpdated="updateFuncRunTab"
          @click="hideFuncRun"
        />
      </template>
    </ul>

    <span class="uppercase font-bold p-xs mt-md">RUN HISTORY</span>
    <template v-for="item in componentManagementHistory" :key="item.funcRunId">
      <ManagementHistoryCard
        :item="item"
        :selected="item.funcRunId === selectedFuncRunId"
        @clickItem="clickItem"
      />
    </template>

    <FuncRunTabGroup
      :close="hideFuncRun"
      :funcRun="funcRun"
      :open="openFuncRunTab"
      :selectedTab="selectedTab"
    />
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import { VormInput } from "@si/vue-lib/design-system";
import { useFuncStore } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";
import {
  ManagementHistoryItem,
  useManagementRunsStore,
} from "@/store/management_runs.store";
import { FuncRun, FuncRunId, useFuncRunsStore } from "@/store/func_runs.store";
import { useViewsStore } from "@/store/views.store";
import ManagementRunPrototype from "./ManagementRunPrototype.vue";
import ManagementHistoryCard from "./Management/ManagementHistoryCard.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "./ModelingDiagram/diagram_types";
import FuncRunTabGroup from "./Actions/FuncRunTabGroup.vue";

const funcStore = useFuncStore();
const funcRunStore = useFuncRunsStore();
const componentsStore = useComponentsStore();
const viewStore = useViewsStore();
const mgmtStore = useManagementRunsStore();

const resourceId = ref("");

const selectedFuncRunId = ref<FuncRunId | undefined>();
const selectedTab = ref<string | undefined>();
const funcRun = ref<FuncRun | undefined>();
const openFuncRunTab = ref(false);

const getFuncRun = async (id: FuncRunId) => {
  if (funcRunStore.funcRuns[id]) {
    funcRun.value = funcRunStore.funcRuns[id];
  } else {
    await funcRunStore.GET_FUNC_RUN(id);
    funcRun.value = funcRunStore.funcRuns[id];
  }
};

const showLatestRunTab = async (id: FuncRunId, slug: string) => {
  openFuncRunTab.value = true;
  selectedFuncRunId.value = id;
  await getFuncRun(id);
  selectedTab.value = slug;
};

const updateFuncRunTab = async (id: FuncRunId) => {
  await getFuncRun(id);
};

const clickItem = async (item: ManagementHistoryItem, _e: MouseEvent) => {
  openFuncRunTab.value = true;
  selectedFuncRunId.value = item.funcRunId;
  await getFuncRun(item.funcRunId);
};

const hideFuncRun = () => {
  openFuncRunTab.value = false;
  selectedFuncRunId.value = undefined;
};

const props = defineProps<{
  component: DiagramGroupData | DiagramNodeData;
}>();

watch(
  () => props.component.def.resourceId,
  () => {
    resourceId.value = props.component.def.resourceId;
  },
  { immediate: true },
);

const saveResource = () => {
  if (viewStore.selectedComponent && resourceId.value)
    componentsStore.SET_RESOURCE_ID(props.component.def.id, resourceId.value);
};

const componentManagementHistory = computed(() =>
  mgmtStore.managementRunHistory.filter(
    (r) => r.componentId === props.component.def.id,
  ),
);
</script>
