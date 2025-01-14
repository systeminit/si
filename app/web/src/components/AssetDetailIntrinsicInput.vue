<template>
  <VormInput
    v-if="display"
    :id="id"
    v-model="display.value"
    :label="label"
    :options="optionsForIntrinsicDisplay"
    compact
    :iconRight="
      display.funcId === identityFuncId ? 'input-socket' : 'circle-slash'
    "
    :disabled="isLocked || display.funcId === unsetFuncId"
    :showCautionLines="display.funcId === unsetFuncId"
    iconRightRotate="down"
    :nullLabel="display.funcId === unsetFuncId ? 'Unset' : 'not set'"
    type="dropdown-optgroup"
    @change="changeInput"
  >
    <template #rightOfInput>
      <DropdownMenu ref="contextMenuRef" :forceAbove="false">
        <DropdownMenuItem
          label="Change Configuration"
          header
          class="uppercase"
        />
        <DropdownMenuItem
          label="Unset"
          checkable
          :checked="selectedFilter === 'unset'"
          @select="selectFilter('unset')"
        />
        <DropdownMenuItem
          label="Bind to Input Socket"
          checkable
          :checked="selectedFilter === 'inputSocket'"
          @select="selectFilter('inputSocket')"
        />
        <DropdownMenuItem
          label="Bind to Prop"
          checkable
          :checked="selectedFilter === 'prop'"
          @select="selectFilter('prop')"
        />
      </DropdownMenu>
      <DetailsPanelMenuIcon
        :disabled="isLocked"
        :selected="contextMenuRef?.isOpen"
        @click="
          (e) => {
            if (!props.isLocked) contextMenuRef?.open(e, false);
          }
        "
      />
    </template>
  </VormInput>
</template>

<script lang="ts" setup>
import { ref, computed, watch, toRaw } from "vue";
import {
  VormInput,
  DropdownMenu,
  DropdownMenuItem,
} from "@si/vue-lib/design-system";
import {
  FuncKind,
  FuncId,
  PropDisplay,
  IntrinsicDisplay,
} from "@/api/sdf/dal/func";
import {
  SchemaVariantId,
  groupedPropsFor,
  inputSocketsFor,
} from "@/api/sdf/dal/schema";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const props = defineProps<{
  schemaVariantId: SchemaVariantId;
  data: PropDisplay | IntrinsicDisplay;
  isLocked: boolean;
}>();

type DropdownFilter = "unset" | "prop" | "inputSocket";

const display = ref<PropDisplay | IntrinsicDisplay | undefined>();

const id = computed<string>(() => {
  if ("socketName" in props.data) return props.data.socketName;
  if ("path" in props.data) return props.data.path;
  return "N/A";
});

const label = computed<string>(() => {
  if ("socketName" in props.data) return props.data.socketName;
  if ("name" in props.data) return props.data.name;
  return "N/A";
});

watch(
  () => props.data,
  () => {
    display.value = toRaw(props.data);
  },
  { immediate: true },
);

const emit = defineEmits(["change", "changeToUnset", "changeToIdentity"]);

const funcStore = useFuncStore();
const assetStore = useAssetStore();

const identityFuncId = computed(() => {
  const func = funcStore.funcList.find(
    (func) => func.kind === FuncKind.Intrinsic && func.name === "si:identity",
  );
  return func?.funcId as FuncId;
});

const unsetFuncId = computed(() => {
  const func = funcStore.funcList.find(
    (func) => func.kind === FuncKind.Intrinsic && func.name === "si:unset",
  );
  return func?.funcId as FuncId;
});

const initialFilter = (): DropdownFilter | null => {
  if (display.value?.value?.startsWith("s_")) {
    return "inputSocket";
  } else if (display.value?.value?.startsWith("p_")) {
    return "prop";
  } else if (display.value?.funcId === unsetFuncId.value) {
    return "unset";
  } else if ("socketName" in props.data) {
    // NOTE(nick): fallback to the input data if the display is empty. We need this in case a "emit" blows the
    // component away. This could be cleaner to avoid having multiple branches.
    return "inputSocket";
  } else if ("path" in props.data) {
    // NOTE(nick): fallback to the input data if the display is empty. We need this in case a "emit" blows the
    // component away. This could be cleaner to avoid having multiple branches.
    return "prop";
  }
  return null;
};
const selectedFilter = ref<DropdownFilter | null>(initialFilter());
const selectFilter = (item: DropdownFilter) => {
  if (props.isLocked) return;

  selectedFilter.value = item;

  if (item === "unset") {
    emit("changeToUnset", display.value);
  } else {
    emit("changeToIdentity", display.value, null);
  }
};

const optionsForIntrinsicDisplay = computed(() => {
  if (!props.schemaVariantId) return {};
  const variant = assetStore.variantFromListById[props.schemaVariantId];
  if (!variant) return {};

  if (selectedFilter.value === "prop") {
    return groupedPropsFor(variant);
  } else if (selectedFilter.value === "inputSocket") {
    return inputSocketsFor(variant);
  }
  return {};
});

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeInput = () => {
  emit("change", toRaw(display.value));
};
</script>
