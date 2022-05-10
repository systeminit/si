<template>
  <div class="flex flex-col justify-center w-full application-card">
    <div class="py-1 pl-2 card-header">
      {{ props.application.name }}
    </div>

    <div class="flex w-full h-full">
      <div class="w-1/5 py-2 mx-2">
        <ApplicationActivitySummary
          :application-id="props.application.id"
          height="40px"
        />
      </div>

      <div class="w-1/5 py-2 mx-2">
        <ApplicationServicesSummary
          :application-id="props.application.id"
          :show-button="false"
        />
      </div>

      <div class="w-1/5 py-2 mx-2">
        <ApplicationComputingResourcesSummary
          :application-id="props.application.id"
          :show-button="false"
        />
      </div>

      <div class="w-1/5 py-2 mx-2">
        <ApplicationProviderSummary :application-id="props.application.id" />
      </div>

      <div class="w-1/5 py-2 mx-2">
        <ApplicationChangesSummary
          :application-id="props.application.id"
          :show-selected-changeset-data="false"
        />
      </div>

      <div class="self-center w-6">
        <VueFeather type="chevron-right" class="button" size="2rem" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Component } from "@/api/sdf/dal/component";
import VueFeather from "vue-feather";
import ApplicationActivitySummary from "./Summary/ApplicationActivitySummary.vue";
import ApplicationServicesSummary from "./Summary/ApplicationServicesSummary.vue";
import ApplicationComputingResourcesSummary from "@/organisims/Application/ApplicationComputingResourcesSummary.vue";
import ApplicationProviderSummary from "@/organisims/Application/Summary/ApplicationProviderSummary.vue";
import ApplicationChangesSummary from "@/organisims/Application/Summary/ApplicationChangesSummary.vue";

const props = defineProps<{
  application: Component;
}>();
</script>

<style scoped>
.application-card {
  @apply shadow-md;
  background-color: #262626;
}

.card-header {
  @apply text-white;
  @apply text-sm;
  @apply font-normal;
  background-color: #3a3d40;
}

.visualization-card {
  @apply bg-gray-900;
  @apply border;
  @apply border-gray-600;
  width: 25%;
}

.visualization-title {
  @apply text-xs;
}

.button {
  color: #4a4b4c;
}

.button:hover {
  @apply text-gray-400;
}

.button:active {
  @apply text-gray-300;
}
</style>
