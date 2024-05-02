<template>
  <div class="overflow-y-auto min-h-[200px]">
    <TreeNode
      v-for="(label, kind) in CUSTOMIZABLE_FUNC_TYPES"
      :key="kind"
      enableDefaultHoverClasses
      enableGroupToggle
      alwaysShowArrow
      indentationSize="none"
      leftBorderSize="none"
      :defaultOpen="funcsForKind(kind).length > 0"
    >
      <template #primaryIcon><FuncSkeleton size="md" /></template>
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
import { PillCounter, TreeNode } from "@si/vue-lib/design-system";
import { Dictionary } from "async";
import SiFuncListItem from "@/components/SiFuncListItem.vue";
import FuncSkeleton from "@/components/FuncSkeleton.vue";
import { FuncSummary } from "@/store/func/funcs.store";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
  customizableFuncKindToFuncKind,
} from "@/api/sdf/dal/func";

const props = defineProps({
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
