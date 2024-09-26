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
    iconRightRotate="down"
    :nullLabel="display.funcId === unsetFuncId ? 'Unset' : 'not set'"
    type="dropdown-optgroup"
    @change="changeInput"
  >
    <template #rightOfInput>
      <DropdownMenu ref="contextMenuRef" :forceAbove="false">
        <DropdownMenuItem header :checkable="false" class="uppercase"
          >Change Configuration</DropdownMenuItem
        >
        <DropdownMenuItem
          :checked="display.funcId === unsetFuncId"
          @select="() => emit('changeToUnset', display)"
          >Unset Binding</DropdownMenuItem
        >
        <DropdownMenuItem
          :checked="display.value?.startsWith('p_')"
          label="Bind to Input Socket"
          :submenuItems="socketSubmenu"
          @select="() => emit('changeToIdentity', display, null)"
        />
        <DropdownMenuItem
          :checked="display.value?.startsWith('s_')"
          label="Bind to Prop"
          :submenuItems="propSubmenu"
          @select="() => emit('changeToIdentity', display, null)"
        />
      </DropdownMenu>
      <DetailsPanelMenuIcon
        :selected="contextMenuRef?.isOpen"
        @click="
          (e) => {
            contextMenuRef?.open(e, false);
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
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import {
  FuncKind,
  FuncId,
  PropDisplay,
  IntrinsicDisplay,
} from "@/api/sdf/dal/func";
import { SchemaVariantId, inputSocketsAndPropsFor } from "@/api/sdf/dal/schema";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const props = defineProps<{
  schemaVariantId: SchemaVariantId;
  data: PropDisplay | IntrinsicDisplay;
  isLocked: boolean;
}>();

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

const optionsForIntrinsicDisplay = computed(() => {
  if (!props.schemaVariantId) return {};
  const variant = assetStore.variantFromListById[props.schemaVariantId];
  if (!variant) return {};
  return inputSocketsAndPropsFor(variant);
});

const socketSubmenu = computed<DropdownMenuItemObjectDef[]>(() => {
  const sockets = optionsForIntrinsicDisplay.value["Input Socket"];
  if (!sockets) return [];
  return sockets.map((socket) => {
    return {
      label: socket.label,
      onSelect: () => select(socket.value as string),
    };
  });
});

const propSubmenu = computed<DropdownMenuItemObjectDef[]>(() => {
  const options: DropdownMenuItemObjectDef[] = [];
  Object.keys(optionsForIntrinsicDisplay.value).forEach((label) => {
    if (label === "Input Socket") return;

    const submenuItems =
      optionsForIntrinsicDisplay.value[label]?.map((option) => {
        const { label, value } = option;
        return {
          label,
          onSelect: () => select(value as string),
        };
      }) || [];
    options.push({
      label,
      disabled: submenuItems.length === 0,
      checkable: false,
      submenuItems,
    });
  });
  return options;
});

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

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeInput = () => {
  emit("change", toRaw(display.value));
};

const select = (value: string) => {
  emit("changeToIdentity", toRaw(display.value), value);
};
</script>
