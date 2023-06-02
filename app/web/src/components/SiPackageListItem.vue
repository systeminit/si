<template>
  <RouterLink
    v-if="packageInfo"
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :class="
      isSelected
        ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
        : ''
    "
    :to="{
      name: 'workspace-lab-packages',
      params: { ...route.params, packageSlug: packageInfo.name },
    }"
  >
    <Icon name="component" />
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ packageInfo.name }}
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import { useRoute } from "vue-router";
import { Icon } from "@si/vue-lib/design-system";
import { ModuleId, useModuleStore } from "../store/module.store";

const props = defineProps({
  packageId: { type: String as PropType<ModuleId>, required: true },
});

const route = useRoute();
const moduleStore = useModuleStore();
const packageInfo = computed(
  () => moduleStore.packageListByName[props.packageId],
);
const isSelected = computed(
  () => packageInfo.value?.name === moduleStore.urlSelectedPackageSlug,
);
</script>
