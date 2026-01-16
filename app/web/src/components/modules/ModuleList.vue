<template>
  <TreeNode
    :label="label"
    defaultOpen
    enableGroupToggle
    alwaysShowArrow
    indentationSize="none"
    leftBorderSize="none"
    enableDefaultHoverClasses
    internalScrolling
    class="min-h-[32px]"
  >
    <ErrorMessage :requestStatus="loading" />
    <div v-if="loading.isPending" class="h-full flex flex-col items-center justify-center">
      <LoadingMessage :message="loadingMessage" :requestStatus="loading" noPadding />
    </div>
    <template v-else-if="loading.isSuccess">
      <div v-if="!modules.length" class="p-sm italic text-center text-xs">
        {{ textSearch ? "No modules match your search" : noModulesMessage }}
      </div>
      <ModuleListItem v-for="p in modules" :key="p.hash" :moduleSlug="p.hash" />
    </template>

    <template #icons>
      <PillCounter :count="modules.length" showHoverInsideTreeNode />
    </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import { ErrorMessage, LoadingMessage, PillCounter, TreeNode } from "@si/vue-lib/design-system";
import { PropType } from "vue";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import { LocalModuleSummary } from "@/store/module.store";
import ModuleListItem from "./ModuleListItem.vue";

defineProps({
  label: { type: String, required: true },
  modules: { type: Array as PropType<LocalModuleSummary[]>, required: true },
  loading: { type: Object as PropType<ApiRequestStatus>, required: true },
  loadingMessage: { type: String, default: "Loading..." },
  textSearch: { type: String },
  noModulesMessage: { type: String, default: "No modules found" },
});
</script>
