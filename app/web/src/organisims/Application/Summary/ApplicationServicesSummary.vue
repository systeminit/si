<template>
  <div class="w-full h-full">
    <SummaryCard>
      <template #title>Services</template>

      <template #content>
        <div class="flex flex-col content-start w-full h-full">
          <div class="flex flex-row flex-wrap w-full h-full mx-1">
            boop
            <!--
            <div
              v-for="resource in servicesData"
              :key="resource.id"
              class="mr-2"
            >
              <ServiceVisualization :resource="resource" />
            </div>
            -->
          </div>

          <div v-show="showButton" class="flex justify-end mt-2">
            <div class="flex text-xs text-center align-middle button">
              <div class="mx-1 button-text">Deploy</div>
            </div>
          </div>
        </div>
      </template>
    </SummaryCard>
  </div>
</template>

<script setup lang="ts">
import SummaryCard from "@/atoms/SummaryCard.vue";

defineProps({
  showButton: {
    type: Boolean,
    default: true,
  },
  applicationId: {
    type: Number,
    required: true,
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.1;

.button {
  background-color: #5a8f91;
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

.button:disabled {
  background-color: red;
  color: red;
}
</style>
