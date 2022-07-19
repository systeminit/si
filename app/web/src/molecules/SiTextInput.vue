<template>
  <div :class="classes">
    <label class="text-xs font-normal block my-1">{{ title }}</label>
    <input
      v-model="value"
      type="text"
      class="w-full p-1 h-7 text-xs rounded-sm border-1 dark:text-black bg-[#FAFAFA] border-[#DBDBDB] placeholder:italic"
      :placeholder="placeholder"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{
  title?: string;
  placeholder?: string;
  icon?: string;
  class?: string;
}>();

const classes = computed(() => props.class);

const value = ref<string>();
</script>
