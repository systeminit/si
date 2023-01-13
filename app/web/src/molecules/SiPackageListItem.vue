<template>
  <RouterLink
    class="flex flex-row items-center gap-2.5 py-4 pr-4 pl-8 text-xs relative border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300"
    :class="
      selectedPackageId === p.id
        ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
        : ''
    "
    :to="{
      name: 'workspace-lab-packages',
      params: { ...route.params, packageSlug: p.slug },
    }"
  >
    <Icon :name="p.icon || 'cat'" />
    <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
      {{ p.displayName }}
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import { useRoute } from "vue-router";
import { storeToRefs } from "pinia";
import Icon from "@/ui-lib/icons/Icon.vue";
import { Package, usePackageStore } from "@/store/package.store";

defineProps({
  p: { type: Object as PropType<Package>, required: true },
});

const route = useRoute();
const packageStore = usePackageStore();
const { selectedPackageId } = storeToRefs(packageStore);
</script>
