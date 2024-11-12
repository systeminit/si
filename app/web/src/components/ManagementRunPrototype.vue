<template>
  <li
    :class="
      clsx(
        'flex flex-row items-center gap-xs p-2xs border-x border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
  >
    <IconButton icon="play" :requestStatus="request" @click="runPrototype()" />
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
import { useComponentsStore } from "@/store/components.store";
import { FuncRunId } from "@/store/func_runs.store";
import { useManagementRunsStore } from "@/store/management_runs.store";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "./ModelingDiagram/diagram_types";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import FuncRunTabDropdown from "./FuncRunTabDropdown.vue";

const funcStore = useFuncStore();
const componentsStore = useComponentsStore();
const router = useRouter();
const toast = useToast();
const managementRunsStore = useManagementRunsStore();

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

watch(latestRunId, (latest) => {
  if (latest) {
    emit("runUpdated", latest);
  }
});

const runPrototype = async () => {
  const result = await funcStore.RUN_MGMT_PROTOTYPE(
    props.prototype.managementPrototypeId,
    props.component.def.id,
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

function onClickView() {
  router.push({
    name: "workspace-lab-assets",
    query: {
      s: `a_${componentsStore.selectedComponent?.def.schemaVariantId}|f_${props.prototype.funcId}`,
    },
  });
}
</script>
