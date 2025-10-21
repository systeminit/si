<template>
  <div class="flex flex-col text-sm text-center">
    <div class="text-destructive-600 dark:text-destructive-500">
      Change set could not be applied...
    </div>
    <div v-if="precondition">
      We are still updating the simulation. Try again momentarily.
    </div>
    <div v-else class="text-destructive-600 dark:text-destructive-500">
      There was a failure on our end. We have been notified. Feel free to try
      again.
    </div>
  </div>
</template>

<script lang="ts" setup>
defineProps<{
  precondition: boolean;
}>();
</script>
