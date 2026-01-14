<template>
  <div v-if="viewStore.selectedComponent" class="flex flex-col h-full w-full">
    <div
      v-if="!props.component.def.resourceId"
      class="text-xs text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
    >
      These functions can require the resource identifier, enter it here
    </div>

    <div class="ml-xs mt-xs">
      <VormInput v-model="resourceId" compact type="text" label="Resource Id" @blur="saveResource" />
    </div>

    <span class="uppercase font-bold p-xs mt-sm">FUNCTION LIST</span>
    <div class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600">
      <div v-if="isLoading">Component update in progress...</div>
      <div v-else>The functions below will run immediately in a change set</div>
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
          @click="hideFuncRun"
        />
      </template>
    </ul>

    <span class="uppercase font-bold p-xs mt-md">RUN HISTORY</span>
    <template v-for="item in componentManagementHistory" :key="item.id">
      <ManagementHistoryCard :item="item" :selected="item.id === selectedFuncRunId" @clickItem="clickItem" />
    </template>

    <FuncRunTabGroup :close="hideFuncRun" :funcRun="funcRun" :open="openFuncRunTab" :selectedTab="selectedTab" />
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import { VormInput } from "@si/vue-lib/design-system";
import { useFuncStore } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";
import { ManagementHistoryItem, useManagementRunsStore } from "@/store/management_runs.store";
import { FuncRunId, useFuncRunsStore } from "@/store/func_runs.store";
import { useViewsStore } from "@/store/views.store";
import { useStatusStore } from "@/store/status.store";
import ManagementRunPrototype from "./ManagementRunPrototype.vue";
import ManagementHistoryCard from "./Management/ManagementHistoryCard.vue";
import { DiagramGroupData, DiagramNodeData } from "./ModelingDiagram/diagram_types";
import FuncRunTabGroup from "./Actions/FuncRunTabGroup.vue";

const funcStore = useFuncStore();
const funcRunStore = useFuncRunsStore();
const componentsStore = useComponentsStore();
const viewStore = useViewsStore();
const mgmtStore = useManagementRunsStore();
const statusStore = useStatusStore();

const resourceId = ref("");

const selectedFuncRunId = ref<FuncRunId | undefined>();
const selectedTab = ref<string | undefined>();
const funcRun = computed(() => {
  if (!selectedFuncRunId.value) return undefined;
  // If it doesn't exist, start a fetch to get it
  if (!funcRunStore.funcRuns[selectedFuncRunId.value]) funcRunStore.GET_FUNC_RUN(selectedFuncRunId.value);
  return funcRunStore.funcRuns[selectedFuncRunId.value];
});
const openFuncRunTab = ref(false);

const showLatestRunTab = async (id: FuncRunId, slug: string) => {
  openFuncRunTab.value = true;
  selectedFuncRunId.value = id;
  selectedTab.value = slug;
};

const clickItem = async (item: ManagementHistoryItem, _e: MouseEvent) => {
  openFuncRunTab.value = true;
  selectedFuncRunId.value = item.id;
};

const hideFuncRun = () => {
  openFuncRunTab.value = false;
  selectedFuncRunId.value = undefined;
};

const props = defineProps<{
  component: DiagramGroupData | DiagramNodeData;
}>();

const isLoading = computed(() => statusStore.componentIsLoading(props.component.def.id));

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
  mgmtStore.managementRunHistory.filter((r) => r.componentId === props.component.def.id),
);
</script>
