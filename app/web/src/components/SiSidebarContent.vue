<template>
  <div class="border-x border-b border-black pt-5 flex flex-col flex-grow bg-white overflow-y-auto">
    <div class="flex-grow m-5 p-5 items-center flex flex-col">
      {{ props.placeholder }}
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  placeholder: string;
}>();
</script>
