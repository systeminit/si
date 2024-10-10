<template>
  <div class="flex flex-col overflow-hidden h-full relative">
    <TreeNode
      v-for="(label, kind) in funcKindOptions"
      :key="kind"
      enableDefaultHoverClasses
      enableGroupToggle
      alwaysShowArrow
      indentationSize="none"
      leftBorderSize="none"
      defaultOpen
      internalScrolling
      class="min-h-[32px]"
      primaryIconClasses=""
    >
      <template #primaryIcon><Icon name="func" :size="'sm'" /></template>
      <template #label>
        <div v-if="label" class="flex items-center gap-xs text-sm">
          <span> {{ label.pluralLabel }} </span>
        </div>
      </template>

      <SiFuncListItem
        v-for="func in funcsForKind(kind)"
        :key="func.funcId"
        :func="func"
        :context="context"
      />
      <div
        v-if="funcsForKind(kind).length === 0"
        class="text-xs w-full text-center italic py-xs text-neutral-500"
      >
        There are no functions of this kind.
      </div>
      <template #icons>
        <PillCounter
          :count="funcsForKind(kind).length"
          showHoverInsideTreeNode
        />
      </template>
    </TreeNode>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { PropType } from "vue";
import { PillCounter, TreeNode, Icon } from "@si/vue-lib/design-system";
import { Dictionary } from "async";
import SiFuncListItem from "@/components/SiFuncListItem.vue";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
  customizableFuncKindToFuncKind,
  FUNC_LABELS,
  FuncSummary,
} from "@/api/sdf/dal/func";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const ffStore = useFeatureFlagsStore();

const funcKindOptions: Partial<Record<CustomizableFuncKind, FUNC_LABELS>> = {};
Object.entries(CUSTOMIZABLE_FUNC_TYPES).forEach(([key, value]) => {
  if (!ffStore.MANAGEMENT_FUNCTIONS && key === CustomizableFuncKind.Management)
    return;
  funcKindOptions[key as CustomizableFuncKind] = value;
});

const props = defineProps({
  defaultOpen: { type: Boolean },
  firstOpen: { type: Boolean },
  context: { type: String, required: true },
  funcsByKind: {
    type: Object as PropType<Dictionary<FuncSummary[]>>,
    default: () => ({}),
  },
});

const funcsForKind = (kind: CustomizableFuncKind) => {
  return props.funcsByKind[customizableFuncKindToFuncKind(kind)] ?? [];
};
</script>
