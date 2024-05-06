<template>
  <TreeNode
    :color="color"
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    labelClasses="w-full"
    noIndentationOrLeftBorder
    :isSelected="storeSelectedFuncId === func.id"
    showSelection
    @mousedown.left.stop="onClick"
  >
    <template #label>
      <div class="w-full flex flex-row gap-xs text-xs justify-between">
        <div class="truncate">
          {{ func.name }}
        </div>
        <SiChip
          :text="chipText"
          :tone="func.isBuiltin ? 'warning' : 'action'"
          uppercase
          variant="simple"
        />
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
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { TreeNode } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import { useFuncStore, FuncSummary } from "@/store/func/funcs.store";
import { trackEvent } from "@/utils/tracking";
import SiChip from "./SiChip.vue";

const props = defineProps<{
  color?: string;
  func: FuncSummary;
  context: string;
}>();

const route = useRoute();
const router = useRouter();
const funcStore = useFuncStore();
const { selectedFuncId: storeSelectedFuncId } = storeToRefs(funcStore);

const chipText = computed(() => (props.func.isBuiltin ? "builtin" : "custom"));

const trackFunctionSelected = () => {
  trackEvent("function_selected_for_edit", {
    func_id: props.func.id,
    func_name: props.func.name,
  });
};

const onClick = () => {
  trackFunctionSelected();
  router.push({
    name: props.context,
    params: { ...route.params, funcId: props.func.id },
  });
};
</script>
