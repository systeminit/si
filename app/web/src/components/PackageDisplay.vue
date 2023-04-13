<template>
  <RequestStatusMessage
    v-if="getPackageReqStatus.isPending"
    :request-status="getPackageReqStatus"
    show-loader-without-message
  />

  <div v-else-if="selectedPackage" class="flex flex-col h-full">
    <div class="flex flex-row items-center gap-2 flex-none">
      <Icon name="component" />
      <div class="text-3xl font-bold truncate">
        {{ selectedPackage.name }}
      </div>
    </div>
    <div
      class="text-sm italic pb-sm flex flex-row flex-wrap gap-x-8 gap-y-1 flex-none"
    >
      <div>
        <span class="font-bold">Version:</span>
        {{ selectedPackage.version }}
      </div>
      <div>
        <span class="font-bold">Created At: </span>
        <Timestamp :date="selectedPackage.createdAt" size="long" />
      </div>
      <div>
        <span class="font-bold">Created By: </span
        >{{ selectedPackage.createdBy }}
      </div>
    </div>
    <div
      class="border dark:border-neutral-600 rounded flex flex-col gap-sm overflow-auto"
    >
      <div
        class="px-sm py-xs border-b dark:border-neutral-600 font-bold flex-none"
      >
        Functions
      </div>

      <ul class="p-sm overflow-y-auto">
        <li
          v-for="func in selectedPackage.funcs"
          :key="func.name"
          class="flex flex-col"
        >
          <div class="flex flex-row items-center">
            <div>
              <i>{{ func.name }}</i>
              <span v-if="func.displayName">: {{ func.displayName }}</span>
            </div>
          </div>
          <div class="pl-lg pb-sm">
            {{ func.description }}
          </div>
        </li>
      </ul>

      <div
        class="px-sm py-xs border-b border-t my-xs dark:border-neutral-600 font-bold flex-none"
      >
        Schema Variants
      </div>

      <ul class="p-sm overflow-y-auto">
        <li
          v-for="sv in selectedPackage.schemas"
          :key="sv"
          class="flex flex-col"
        >
          <div class="flex flex-row items-center">
            <div>{{ sv }}</div>
          </div>
          <div class="pl-lg pb-sm">other info goes here</div>
        </li>
      </ul>
    </div>
  </div>
  <div
    v-else-if="
      packageStore.urlSelectedPackageSlug && getPackageReqStatus.isError
    "
    class="text-neutral-400 dark:text-neutral-300 text-center"
  >
    <ErrorMessage :request-status="getPackageReqStatus" />
  </div>
  <div v-else class="text-neutral-400 dark:text-neutral-300 text-center">
    <ErrorMessage v-if="packageStore.urlSelectedPackageSlug">
      Package "{{ packageStore.urlSelectedPackageSlug }}" does not exist!
    </ErrorMessage>
    <template v-else>Select a package to view it.</template>
  </div>
</template>

<script lang="ts" setup>
import { computed, watch } from "vue";
import {
  Icon,
  RequestStatusMessage,
  Timestamp,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { usePackageStore } from "@/store/package.store";

const packageStore = usePackageStore();
const getPackageReqStatus = packageStore.getRequestStatus("GET_PACKAGE");

const selectedPackage = computed(() => packageStore.selectedPackage);
const selectedPackageListItem = computed(
  () => packageStore.selectedPackageListItem,
);

watch(
  () => packageStore.urlSelectedPackageSlug,
  (selectedPackageName) => {
    if (selectedPackageName && selectedPackageListItem.value) {
      packageStore.GET_PACKAGE(selectedPackageListItem.value);
    }
  },
);
</script>
