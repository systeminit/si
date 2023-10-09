<template>
  <RouterLink
    v-if="moduleSummary"
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :class="
      isSelected
        ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
        : ''
    "
    :to="{
      name: 'workspace-lab-packages',
      params: {
        ...route.params,
        moduleSlug: moduleSummary.hash,
      },
    }"
  >
    <Icon name="component" />
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ moduleSummary.name }}
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import { RouterLink, useRoute } from "vue-router";
import { Icon } from "@si/vue-lib/design-system";
import { ModuleSlug, useModuleStore } from "@/store/module.store";

const props = defineProps({
  moduleSlug: { type: String as PropType<ModuleSlug>, required: true },
});

const route = useRoute();
const moduleStore = useModuleStore();
const moduleSummary = computed(() => {
  return (
    moduleStore.localModulesByHash[props.moduleSlug] ||
    moduleStore.remoteModuleSummaryByHash[props.moduleSlug] ||
    moduleStore.builtinModuleSummaryByHash[props.moduleSlug]
  );
});
const isSelected = computed(
  () => moduleSummary.value?.hash === moduleStore.urlSelectedModuleSlug,
);
</script>
