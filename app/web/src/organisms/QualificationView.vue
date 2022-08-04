<template>
  <div class="border rounded-xl py-2 px-5 mx-4 my-4 text-center">
    <h1 class="text-xl mb-2">
      <CheckCircleIcon class="text-success-500 w-6 inline align-middle mr-1" />
      <span class="align-middle">{{ props.qualification.title }}</span>
    </h1>
    <p class="text-sm">{{ props.qualification }}</p>
  </div>
</template>

<script setup lang="ts">
import { Qualification } from "@/api/sdf/dal/qualification";
import { CheckCircleIcon } from "@heroicons/vue/outline";

const props = defineProps<{
  qualification: Qualification;
}>();
</script>
