<template>
  <TreeNode
    v-if="moduleSummary"
    :classes="
      clsx(
        'dark:text-white text-black dark:bg-neutral-800 py-[1px]',
        'hover:dark:outline-action-300 hover:outline-action-500 hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
      )
    "
    showSelection
    labelClasses="w-full"
    leftBorderSize="none"
    primaryIconClasses=""
    :isSelected="isSelected"
    @mousedown.left.stop="onClick"
  >
    <template #primaryIcon><Icon name="component" size="sm" /></template>
    <template #label>
      <div class="w-full truncate text-xs">
        {{ moduleSummary.name }}
      </div>
    </template>
  </TreeNode>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Icon, TreeNode } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ModuleSlug, useModuleStore } from "@/store/module.store";

const props = defineProps({
  moduleSlug: { type: String as PropType<ModuleSlug>, required: true },
});

const route = useRoute();
const router = useRouter();
const moduleStore = useModuleStore();
const moduleSummary = computed(() => {
  return (
    moduleStore.localModulesByHash[props.moduleSlug] ||
    moduleStore.remoteModuleSummaryByHash[props.moduleSlug] ||
    moduleStore.builtinModuleSummaryByHash[props.moduleSlug]
  );
});
const isSelected = computed(() => moduleSummary.value?.hash === moduleStore.urlSelectedModuleSlug);

const onClick = () => {
  if (moduleSummary.value) {
    router.push({
      name: "workspace-lab-packages",
      params: {
        ...route.params,
        moduleSlug: moduleSummary.value.hash,
      },
    });
  }
};
</script>
