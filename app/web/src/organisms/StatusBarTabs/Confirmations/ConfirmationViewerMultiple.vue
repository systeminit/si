<template>
  <div class="h-full flex-grow flex flex-col bg-shade-100 min-w-0">
    <div class="overflow-y-auto flex flex-row mt-4 mx-2 flex-wrap">
      <div
        v-for="(confirmation, index) in resource.confirmations"
        :key="index"
        class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-4 px-2"
      >
        <ConfirmationViewerSingle :confirmation="confirmation" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { MockResource } from "@/service/resource";
import ConfirmationViewerSingle from "./ConfirmationViewerSingle.vue";

defineProps<{
  resource: MockResource;
}>();
</script>
