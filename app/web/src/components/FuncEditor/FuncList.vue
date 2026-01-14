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
      :defaultOpen="funcsForKind(kind as CustomizableFuncKind).length > 0"
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
        v-for="func in funcsForKind(kind as CustomizableFuncKind)"
        :key="func.funcId"
        :func="func"
        :context="context"
      />
      <div
        v-if="funcsForKind(kind as CustomizableFuncKind).length === 0"
        class="text-xs w-full text-center italic py-xs text-neutral-500"
      >
        There are no functions of this kind.
      </div>
      <template #icons>
        <PillCounter :count="funcsForKind(kind as CustomizableFuncKind).length" showHoverInsideTreeNode />
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
  FUNC_TYPES,
  FuncSummary,
} from "@/api/sdf/dal/func";

// filtering out a func type if FF for mgmt functions is off
// When you use an enum as keys in a record
// TS errors out if all enum values are not in the record
// but that's literally not what I want! because I want to delete one!
// So, I can Partial to make all keys optional
const funcKindOptions: Partial<FUNC_TYPES> = {};
Object.entries(CUSTOMIZABLE_FUNC_TYPES).forEach(([key, value]) => {
  funcKindOptions[key] = value;
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
