<template>
  <RouterLink
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :class="
      storeSelectedFuncId === func.id
        ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
        : ''
    "
    :to="{
      name: context,
      params: { ...route.params, funcId: func.id },
    }"
    @click="trackFunctionSelected()"
  >
    <div class="w-full truncate">
      {{ func.name }}
    </div>
    <SiChip
      :text="func.isBuiltin ? 'builtin' : 'custom'"
      :variant="func.isBuiltin ? 'warning' : 'neutral'"
      class="right-4"
    />
  </RouterLink>
</template>

<script setup lang="ts">
import { RouterLink, useRoute } from "vue-router";
import { storeToRefs } from "pinia";
import { useFuncStore, FuncSummary } from "@/store/func/funcs.store";
import { trackEvent } from "@/utils/tracking";
import SiChip from "./SiChip.vue";

const props = defineProps<{
  color?: string;
  func: FuncSummary;
  context: string;
}>();

const route = useRoute();
const funcStore = useFuncStore();
const { selectedFuncId: storeSelectedFuncId } = storeToRefs(funcStore);
const trackFunctionSelected = () => {
  trackEvent("function_selected_for_edit", {
    func_id: props.func.id,
    func_name: props.func.name,
  });
};
</script>
