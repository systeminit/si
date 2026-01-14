<template>
  <li
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
  >
    <DropdownMenu ref="viewSelectorMenuRef" forceAlignRight>
      <DropdownMenuItem header label="RUN IN VIEW" />
      <DropdownMenuItem
        v-for="viewId in componentViews"
        :key="viewId"
        :label="viewsStore.viewsById[viewId]?.name ?? 'unknown'"
        :onSelect="() => runPrototype(viewId)"
      />
    </DropdownMenu>

    <IconButton
      icon="play"
      :requestStatus="request"
      :disabled="isLoading"
      :tooltip="isLoading ? `Wait for component finish updating` : `Run Function`"
      @click="runClick"
    />

    <TruncateWithTooltip class="grow">{{ `Run ${props.prototype.label}` }}</TruncateWithTooltip>
    <StatusIndicatorIcon v-if="lastExecutionState" type="management" :status="lastExecutionState" />

    <FuncRunTabDropdown
      :funcRunId="latestRunId"
      showFuncView
      @viewFunc="onClickView"
      @menuClick="(id, slug) => emit('showLatestRunTab', id, slug)"
    />
  </li>
</template>

<script lang="ts" setup>
import { ref, computed, onMounted } from "vue";
import clsx from "clsx";
import {
  DropdownMenu,
  DropdownMenuItem,
  IconButton,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useFuncStore, MgmtPrototype } from "@/store/func/funcs.store";
import { FuncRun, FuncRunId, funcRunStatus, useFuncRunsStore } from "@/store/func_runs.store";
import { useManagementRunsStore } from "@/store/management_runs.store";
import { useStatusStore } from "@/store/status.store";
import { useViewsStore } from "@/store/views.store";
import { ViewId } from "@/api/sdf/dal/views";
import { ComponentType } from "@/api/sdf/dal/schema";
import { DiagramGroupData, DiagramNodeData } from "./ModelingDiagram/diagram_types";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FuncRunTabDropdown from "./FuncRunTabDropdown.vue";

const funcStore = useFuncStore();
const viewStore = useViewsStore();
const router = useRouter();
const statusStore = useStatusStore();
const managementRunsStore = useManagementRunsStore();
const viewsStore = useViewsStore();
const funcRunStore = useFuncRunsStore();

const viewSelectorMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const props = defineProps<{
  prototype: MgmtPrototype;
  component: DiagramGroupData | DiagramNodeData;
}>();

const emit = defineEmits<{
  (e: "showLatestRunTab", id: FuncRunId, slug: string): void;
}>();

const request = funcStore.getRequestStatus(
  "RUN_MGMT_PROTOTYPE",
  props.prototype.managementPrototypeId,
  props.component.def.id,
);

const isLoading = computed(() => statusStore.componentIsLoading(props.component.def.id));

const historicalFuncRun = ref<FuncRun | null>(null);

onMounted(async () => {
  const resp = await managementRunsStore.GET_LATEST_FOR_MGMT_PROTO_AND_COMPONENT(
    props.prototype.managementPrototypeId,
    props.component.def.id,
  );
  if (resp.result.success) {
    historicalFuncRun.value = resp.result.data;
  }
});

const latestRunId = computed(() =>
  managementRunsStore.latestManagementRun(props.prototype.managementPrototypeId, props.component.def.id),
);

const componentViews = computed(() =>
  Object.keys(viewsStore.viewsById).filter(
    (viewId) =>
      !!viewsStore.viewsById[viewId]?.components[props.component.def.id] ||
      !!viewsStore.viewsById[viewId]?.groups[props.component.def.id],
  ),
);

const lastExecution = computed<FuncRun | null>(() => {
  if (latestRunId.value) {
    const r = funcRunStore.funcRuns[latestRunId.value];
    if (!r) return null;
    return r;
  } else {
    return historicalFuncRun.value;
  }
});

const lastExecutionState = computed(() => funcRunStatus(lastExecution.value));
const runPrototype = async (viewId: ViewId) => {
  funcStore.RUN_MGMT_PROTOTYPE(props.prototype.managementPrototypeId, props.component.def.id, viewId);
};

const runClick = async (e?: MouseEvent) => {
  if (componentViews.value.length === 1 && componentViews.value[0]) {
    await runPrototype(componentViews.value[0]);
  } else if (componentViews.value.length > 1) {
    viewSelectorMenuRef.value?.open(e, false);
  } else if (viewsStore.selectedViewId) {
    await runPrototype(viewsStore.selectedViewId);
  }
};

function onClickView() {
  if (viewStore.selectedComponent?.def.componentType !== ComponentType.View)
    router.push({
      name: "workspace-lab-assets",
      query: {
        s: `a_${viewStore.selectedComponent?.def.schemaVariantId}|f_${props.prototype.funcId}`,
      },
    });
}
</script>
