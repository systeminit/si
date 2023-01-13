<template>
  <div v-if="packageStore.selectedPackage" class="p-sm flex flex-col">
    <div class="flex flex-row items-center gap-2">
      <Icon :name="packageStore.selectedPackage.icon" />
      <div class="text-3xl font-bold">
        {{ packageStore.selectedPackage.displayName }}
      </div>
    </div>
    <div class="text-sm italic pb-sm">
      Version: {{ packageStore.selectedPackage.version }}
    </div>
    <div class="border dark:border-neutral-600 rounded">
      <div class="px-sm py-xs border-b dark:border-neutral-600 font-bold">
        Schema Variants
      </div>
      <ul class="p-sm">
        <li
          v-for="sv in packageStore.selectedPackage.schemaVariants"
          :key="sv.id"
        >
          {{ sv.name }}
        </li>
      </ul>
    </div>
  </div>
  <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    Select a package to view it.
  </div>
</template>

<script lang="ts" setup>
import { usePackageStore } from "@/store/package.store";
import Icon from "@/ui-lib/icons/Icon.vue";

const packageStore = usePackageStore();
</script>
