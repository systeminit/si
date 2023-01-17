<template>
  <div
    v-if="
      packageStore.selectedPackage && packageStore.selectedPackage.slug === slug
    "
    class="p-sm flex flex-col h-full"
  >
    <div
      class="flex flex-row items-center gap-2 flex-none"
      :style="`color: ${packageStore.selectedPackage.color}`"
    >
      <Icon :name="packageStore.selectedPackage.icon" />
      <div class="text-3xl font-bold truncate">
        {{ packageStore.selectedPackage.displayName }}
      </div>
    </div>
    <div
      class="text-sm italic pb-sm flex flex-row flex-wrap gap-x-8 gap-y-1 flex-none"
    >
      <div>
        <span class="font-bold">Version:</span>
        {{ packageStore.selectedPackage.version }}
      </div>
      <div>
        <span class="font-bold">Created At: </span>
        <Timestamp :date="packageStore.selectedPackage.createdAt" size="long" />
      </div>
      <div>
        <span class="font-bold">Created By: </span
        >{{ packageStore.selectedPackage.createdBy }}
      </div>
    </div>
    <div
      class="border dark:border-neutral-600 rounded flex flex-col overflow-auto"
    >
      <div
        class="px-sm py-xs border-b dark:border-neutral-600 font-bold flex-none"
      >
        Schema Variants
      </div>

      <ul class="p-sm overflow-y-auto">
        <li
          v-for="sv in packageStore.selectedPackage.schemaVariants"
          :key="sv.id"
          class="flex flex-col"
        >
          <div class="flex flex-row items-center">
            <div :style="`color: ${sv.color}`">
              <Icon :name="packageStore.selectedPackage.icon" />
            </div>
            <div>{{ sv.name }}</div>
          </div>
          <div class="pl-lg pb-sm">other info goes here</div>
        </li>
      </ul>
    </div>
  </div>
  <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    <template v-if="slug">Package "{{ slug }}" does not exist!</template>
    <template v-else>Select a package to view it.</template>
  </div>
</template>

<script lang="ts" setup>
import { usePackageStore } from "@/store/package.store";
import Icon from "@/ui-lib/icons/Icon.vue";
import Timestamp from "@/ui-lib/Timestamp.vue";

const packageStore = usePackageStore();

defineProps<{
  slug?: string;
}>();
</script>
