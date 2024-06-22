<template>
  <TreeNode
    :color="color"
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    noIndentationOrLeftBorder
    :isSelected="funcStore.selectedFuncId === func.id"
    showSelection
    @mousedown.left.stop="onClick"
  >
    <template #label>
      <div class="w-full flex flex-row gap-xs text-xs justify-between">
        <div class="truncate">
          {{ func.name }}
        </div>
      </div>
      <!-- <div
                class="italic text-xs text-neutral-500 dark:text-neutral-400"
              >
                asset by: System Initiative
              </div> -->
    </template>
  </TreeNode>
</template>

<script setup lang="ts">
import { TreeNode } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { FuncSummary, useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { trackEvent } from "@/utils/tracking";

const props = defineProps<{
  color?: string;
  func: FuncSummary;
  context: string;
}>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();

const trackFunctionSelected = () => {
  trackEvent("function_selected_for_edit", {
    func_id: props.func.id,
    func_name: props.func.name,
  });
};

const onClick = () => {
  trackFunctionSelected();
  assetStore.addFuncSelection(props.func.id);
};
</script>
