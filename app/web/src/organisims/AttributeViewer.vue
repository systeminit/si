<template>
  <div v-if="componentId" class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">TODO: componmentType</div>
    </div>
    <EditFormComponent :object-id="props.componentId" />
  </div>
</template>

<script setup lang="ts">
import EditFormComponent from "@/organisims/EditFormComponent.vue";

const props = defineProps<{
  componentId: number;
}>();
</script>

<style scoped>
.scrollbar {
  -ms-overflow-style: none; /* edge, and ie */
  scrollbar-width: none; /* firefox */
}

.scrollbar::-webkit-scrollbar {
  display: none; /*chrome, opera, and safari */
}

.property-section-bg-color {
  background-color: #292c2d;
}
</style>
