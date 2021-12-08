<template>
  <SummaryCard>
    <template #title>Providers</template>

    <template #content>
      <div class="flex flex-col w-full h-full">
        <div class="flex flex-row flex-wrap w-full h-full mx-1">
          <!--
          <div class="mr-2" v-for="resource in providerData" :key="resource.id">
            <ResourceVisualization :resource="resource" />
          </div>
          -->
        </div>
      </div>
    </template>
  </SummaryCard>
</template>

<script setup lang="ts">
import SummaryCard from "@/atoms/SummaryCard.vue";

defineProps({
  applicationId: {
    type: Number,
    required: true,
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.05;

.details-panel {
  border: 1px solid #464753;
  background-color: #101010;
}

.details-panel-title {
  /* @apply font-normal text-xs; */
  font-weight: 400;
  font-size: 0.75rem;
  line-height: 1rem;
}

.button {
  background-color: #5a7b7c;
}

.button-text {
  @apply font-normal;
  font-size: 11px;
  margin-top: 2px;
  margin-bottom: 2px;
}

.button:hover {
  filter: brightness($button-brightness);
}

.button:focus {
  outline: none;
}

.button:active {
  filter: saturate(1.5) brightness($button-brightness);
}
</style>
