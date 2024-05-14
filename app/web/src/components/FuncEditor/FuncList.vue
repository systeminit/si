<template>
  <div class="flex flex-col overflow-hidden h-full relative">
    <TreeNode
      v-for="(label, kind, index) in CUSTOMIZABLE_FUNC_TYPES"
      :key="kind"
      enableDefaultHoverClasses
      enableGroupToggle
      alwaysShowArrow
      indentationSize="none"
      leftBorderSize="none"
      :defaultOpen="
        (defaultOpen || (index === 0 && firstOpen)) &&
        !(funcsForKind(kind).length <= 0)
      "
      internalScrolling
      class="min-h-[32px]"
      primaryIconClasses=""
    >
      <template #primaryIcon><Icon name="func" :size="'sm'" /></template>
      <template #label>
        <div class="flex items-center gap-xs text-sm">
          <span> {{ label.pluralLabel }} </span>
        </div>
      </template>

      <SiFuncListItem
        v-for="func in funcsForKind(kind)"
        :key="func.id"
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
import { FuncSummary } from "@/store/func/funcs.store";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
  customizableFuncKindToFuncKind,
} from "@/api/sdf/dal/func";

const props = defineProps({
  defaultOpen: { type: Boolean },
  firstOpen: { type: Boolean },
  context: { type: String, default: "workspace-lab-functions" },
  funcsByKind: {
    type: Object as PropType<Dictionary<FuncSummary[]>>,
    default: () => ({}),
  },
});

const funcsForKind = (kind: CustomizableFuncKind) => {
  return props.funcsByKind[customizableFuncKindToFuncKind(kind)] ?? [];
};
</script>
