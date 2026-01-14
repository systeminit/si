<template>
  <VormInput
    v-if="display"
    :id="id"
    v-model="display.value"
    :labelTooltip="tooltipValue"
    :label="label"
    :options="optionsForIntrinsicDisplay"
    compact
    :iconRight="icon"
    :disabled="isLocked || display.funcId === unsetFuncId"
    :showCautionLines="display.funcId === unsetFuncId"
    iconRightRotate="down"
    :nullLabel="display.funcId === unsetFuncId ? 'Unset' : 'not set'"
    type="dropdown-optgroup"
    @change="changeInput"
  >
    <template #rightOfInput>
      <DropdownMenu ref="contextMenuRef" :forceAbove="false">
        <DropdownMenuItem label="Change Configuration" header class="uppercase" />
        <DropdownMenuItem
          label="Unset"
          checkable
          :checked="selectedFilter === 'unset'"
          @select="selectFilter('unset')"
        />
        <span class="pl-xs text-neutral-500">Identity</span>
        <DropdownMenuItem
          label="Bind to Input Socket"
          checkable
          :checked="selectedFilter === 'inputSocketForIdentity'"
          @select="selectFilter('inputSocketForIdentity')"
        />
        <DropdownMenuItem
          label="Bind to Prop"
          checkable
          :checked="selectedFilter === 'propForIdentity'"
          @select="selectFilter('propForIdentity')"
        />
        <span v-if="normalizeToArrayFuncId" class="flex pl-xs text-neutral-500">Normalize to Array</span>
        <span v-else class="flex pl-xs text-neutral-500">Normalize to Array (Regenerate to Install)</span>
        <DropdownMenuItem
          label="Bind to Input Socket"
          checkable
          :disabled="!normalizeToArrayFuncId"
          :checked="selectedFilter === 'inputSocketForNormalizeToArray'"
          @select="selectFilter('inputSocketForNormalizeToArray')"
        />
        <DropdownMenuItem
          label="Bind to Prop"
          checkable
          :disabled="!normalizeToArrayFuncId"
          :checked="selectedFilter === 'propForNormalizeToArray'"
          @select="selectFilter('propForNormalizeToArray')"
        />
      </DropdownMenu>
      <DetailsPanelMenuIcon
        :disabled="isLocked"
        :selected="contextMenuRef?.isOpen"
        @click="
          (e: MouseEvent) => {
            if (!props.isLocked) contextMenuRef?.open(e, false);
          }
        "
      />
    </template>
  </VormInput>
</template>

<script lang="ts" setup>
import { ref, computed, watch, toRaw } from "vue";
import { VormInput, DropdownMenu, DropdownMenuItem } from "@si/vue-lib/design-system";
import { FuncKind, FuncId, PropDisplay, IntrinsicDisplay } from "@/api/sdf/dal/func";
import { SchemaVariantId, groupedPropsFor, inputSocketsFor } from "@/api/sdf/dal/schema";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const props = defineProps<{
  schemaVariantId: SchemaVariantId;
  data: PropDisplay | IntrinsicDisplay;
  isLocked: boolean;
}>();

type DropdownFilter =
  | "unset"
  | "propForIdentity"
  | "inputSocketForIdentity"
  | "propForNormalizeToArray"
  | "inputSocketForNormalizeToArray";

const display = ref<PropDisplay | IntrinsicDisplay | undefined>();

const id = computed<string>(() => {
  if ("socketName" in props.data) return props.data.socketName;
  if ("path" in props.data) return props.data.path;
  return "N/A";
});

const label = computed<string>(() => {
  if ("socketName" in props.data) return props.data.socketName;
  if ("name" in props.data) return props.data.path.replace("/root", "");
  return "N/A";
});

const tooltipValue = computed<string>(() => {
  if ("name" in props.data) return props.data.path;
  return "";
});

watch(
  () => props.data,
  () => {
    // we need to make a shallow copy of props.data
    // in order to prevent the parent AssetDetailsPanel from
    // recomputing outputSocketIntrinsics which causes this component
    // to be unmounted/mounted which resets the selectedFilter :(
    display.value = { ...toRaw(props.data) };
  },
  { immediate: true },
);

const emit = defineEmits(["change", "changeIntrinsicFunc"]);

const funcStore = useFuncStore();
const assetStore = useAssetStore();

const identityFuncId = computed(() => {
  const func = funcStore.funcList.find((func) => func.kind === FuncKind.Intrinsic && func.name === "si:identity");
  return func?.funcId as FuncId;
});

const normalizeToArrayFuncId = computed(() => {
  const func = funcStore.funcList.find(
    (func) => func.kind === FuncKind.Intrinsic && func.name === "si:normalizeToArray",
  );
  return func?.funcId as FuncId;
});

const unsetFuncId = computed(() => {
  const func = funcStore.funcList.find((func) => func.kind === FuncKind.Intrinsic && func.name === "si:unset");
  return func?.funcId as FuncId;
});

const icon = computed(() => {
  if (display.value?.funcId === identityFuncId.value) return "input-socket";
  else if (display.value?.funcId === normalizeToArrayFuncId.value) return "brackets-square";
  return "circle-slash";
});

const initialFilter = (): DropdownFilter | null => {
  if (display.value?.value?.startsWith("s_")) {
    if (display.value?.funcId === normalizeToArrayFuncId.value) return "inputSocketForNormalizeToArray";
    return "inputSocketForIdentity";
  } else if (display.value?.value?.startsWith("p_")) {
    if (display.value?.funcId === normalizeToArrayFuncId.value) return "propForNormalizeToArray";
    return "propForIdentity";
  } else if (display.value?.funcId === unsetFuncId.value) {
    return "unset";
  } else if ("socketName" in props.data) {
    // NOTE(nick): fallback to the input data if the display is empty. We need this in case a "emit" blows the
    // component away. This could be cleaner to avoid having multiple branches.
    return "inputSocketForIdentity";
  } else if ("path" in props.data) {
    // NOTE(nick): fallback to the input data if the display is empty. We need this in case a "emit" blows the
    // component away. This could be cleaner to avoid having multiple branches.
    return "propForIdentity";
  }
  return null;
};

const selectedFilter = ref<DropdownFilter | null>(initialFilter());
const selectFilter = (item: DropdownFilter) => {
  if (props.isLocked) return;

  selectedFilter.value = item;

  if (item === "unset") {
    emit("changeIntrinsicFunc", "unset", display.value);
  } else if (item === "propForNormalizeToArray" || item === "inputSocketForNormalizeToArray") {
    emit("changeIntrinsicFunc", "normalizeToArray", display.value);
  } else {
    emit("changeIntrinsicFunc", "identity", display.value);
  }
};

const optionsForIntrinsicDisplay = computed(() => {
  if (!props.schemaVariantId) return {};
  const variant = assetStore.variantFromListById[props.schemaVariantId];
  if (!variant) return {};

  if (selectedFilter.value === "propForIdentity" || selectedFilter.value === "propForNormalizeToArray") {
    return groupedPropsFor(variant);
  } else if (
    selectedFilter.value === "inputSocketForIdentity" ||
    selectedFilter.value === "inputSocketForNormalizeToArray"
  ) {
    return inputSocketsFor(variant);
  }
  return {};
});

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeInput = () => {
  emit("change", toRaw(display.value));
};
</script>
