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

    <IconButton icon="play" :requestStatus="request" @click="runClick" />

    <TruncateWithTooltip class="grow">{{
      `Run ${props.prototype.label}`
    }}</TruncateWithTooltip>
    <StatusIndicatorIcon
      v-if="lastExecution"
      type="management"
      :status="lastExecution.status"
    />

    <FuncRunTabDropdown
      :funcRunId="latestRunId"
      showFuncView
      @viewFunc="onClickView"
      @menuClick="(id, slug) => emit('showLatestRunTab', id, slug)"
    />
  </li>
</template>

<script lang="ts" setup>
import { ref, computed, onMounted, watch } from "vue";
import clsx from "clsx";
import { useToast } from "vue-toastification";
import {
  DropdownMenu,
  DropdownMenuItem,
  IconButton,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import {
  useFuncStore,
  MgmtPrototype,
  MgmtPrototypeResult,
} from "@/store/func/funcs.store";
import { FuncRunId } from "@/store/func_runs.store";
import { useManagementRunsStore } from "@/store/management_runs.store";
import { useViewsStore } from "@/store/views.store";
import { ViewId } from "@/api/sdf/dal/views";
import { ComponentType } from "@/api/sdf/dal/schema";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "./ModelingDiagram/diagram_types";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FuncRunTabDropdown from "./FuncRunTabDropdown.vue";

const funcStore = useFuncStore();
const viewStore = useViewsStore();
const router = useRouter();
const toast = useToast();
const managementRunsStore = useManagementRunsStore();
const viewsStore = useViewsStore();

const viewSelectorMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const lastExecution = ref<MgmtPrototypeResult | undefined>(undefined);

const props = defineProps<{
  prototype: MgmtPrototype;
  component: DiagramGroupData | DiagramNodeData;
}>();

const emit = defineEmits<{
  (e: "showLatestRunTab", id: FuncRunId, slug: string): void;
  (e: "runUpdated", id: FuncRunId): void;
}>();

const request = funcStore.getRequestStatus(
  "RUN_MGMT_PROTOTYPE",
  props.prototype.managementPrototypeId,
  props.component.def.id,
);

onMounted(() => {
  managementRunsStore.GET_LATEST_FOR_MGMT_PROTO_AND_COMPONENT(
    props.prototype.managementPrototypeId,
    props.component.def.id,
  );
});

const latestRunId = computed(() =>
  managementRunsStore.latestManagementRun(
    props.prototype.managementPrototypeId,
    props.component.def.id,
  ),
);

const componentViews = computed(() =>
  Object.keys(viewsStore.viewsById).filter(
    (viewId) =>
      !!viewsStore.viewsById[viewId]?.components[props.component.def.id] ||
      !!viewsStore.viewsById[viewId]?.groups[props.component.def.id],
  ),
);

watch(latestRunId, (latest) => {
  if (latest) {
    emit("runUpdated", latest);
  }
});

const runPrototype = async (viewId: ViewId) => {
  const result = await funcStore.RUN_MGMT_PROTOTYPE(
    props.prototype.managementPrototypeId,
    props.component.def.id,
    viewId,
  );

  if (result.result.success) {
    lastExecution.value = result.result.data;
    if (result.result.data.message) {
      const toastOptions = {
        pauseOnHover: true,
        timeout: 5000,
      };
      if (result.result.data.status === "ok") {
        toast.success(
          `${props.prototype.label}: ${result.result.data.message}`,
          toastOptions,
        );
      } else {
        toast.warning(
          `${props.prototype.label}: ${result.result.data.message}`,
          toastOptions,
        );
      }
    }
  }
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
