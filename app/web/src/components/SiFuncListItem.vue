<template>
  <TreeNode
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    :color="color"
    :isSelected="funcStore.selectedFuncId === func.funcId"
    noIndentationOrLeftBorder
    showSelection
    @mousedown.left.stop="onClick"
  >
    <template #label>
      <div
        class="w-full flex flex-row gap-xs text-xs justify-between items-center"
      >
        <div class="truncate">
          {{ func.name }}
        </div>
        <StatusIndicatorIcon
          v-if="changeSetStore.functionConflicts.includes(func.funcId)"
          v-tooltip="{
            content: 'Conflict',
            theme: 'instant-show',
          }"
          size="sm"
          type="conflict"
          class="hover:scale-110"
        />
        <EditingPill v-if="!func.isLocked" color="#666"></EditingPill>
      </div>
      <!-- <div
                class="italic text-xs text-neutral-500 dark:text-neutral-400"
              >
                asset by: System Initiative
              </div> -->
    </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import { TreeNode } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useFuncStore } from "@/store/func/funcs.store";
import { FuncSummary } from "@/api/sdf/dal/func";
import { useAssetStore } from "@/store/asset.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { trackEvent } from "@/utils/tracking";
import EditingPill from "@/components/EditingPill.vue";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";

const props = defineProps<{
  color?: string;
  func: FuncSummary;
  context: string;
}>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();
const changeSetStore = useChangeSetsStore();

const trackFunctionSelected = () => {
  trackEvent("function_selected_for_edit", {
    func_id: props.func.funcId,
    func_name: props.func.name,
  });
};

const onClick = () => {
  trackFunctionSelected();
  assetStore.setFuncSelection(props.func.funcId);
};
</script>
