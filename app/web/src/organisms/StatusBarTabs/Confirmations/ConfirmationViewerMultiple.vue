<template>
  <div class="h-full flex-grow flex flex-col bg-shade-100 min-w-0">
    <div class="overflow-y-auto flex flex-row mt-4 mx-2 flex-wrap">
      <div
        v-for="(confirmation, index) in selectedResource.confirmations"
        :key="index"
        class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-4 px-2"
      >
        <ConfirmationViewerSingle :confirmation="confirmation" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { MockResource } from "@/store/resources.store";
import ConfirmationViewerSingle from "./ConfirmationViewerSingle.vue";

defineProps<{
  // NOTE - the prop name `resource` was causing weird issues as if it had some built-in/reserved meaning?
  selectedResource: MockResource;
}>();
</script>
