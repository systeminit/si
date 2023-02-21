<template>
  <RouterLink
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :class="
      isSelected
        ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
        : ''
    "
    :to="{
      name: 'workspace-lab-packages',
      params: { ...route.params, packageSlug: packageInfo.slug },
    }"
  >
    <Icon :name="packageInfo.icon || 'cat'" />
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ packageInfo.displayName }}
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import { useRoute } from "vue-router";
import Icon from "@/ui-lib/icons/Icon.vue";
import { PackageId, usePackageStore } from "@/store/package.store";

const props = defineProps({
  packageId: { type: String as PropType<PackageId>, required: true },
});

const route = useRoute();
const packageStore = usePackageStore();
const packageInfo = computed(() => packageStore.packagesById[props.packageId]);
const isSelected = computed(
  () => packageInfo.value.slug === packageStore.urlSelectedPackageSlug,
);
</script>
