<template>
  <div
    class="flex nowrap gap-3 p-3 dark:border-neutral-600 border-b-2 justify-between"
  >
    <label
      class="relative text-neutral-400 focus-within:text-gray-600 block flex-grow"
    >
      <input
        v-model="search"
        :placeholder="placeholder"
        class="w-full px-1 py-[0.4375rem] pl-2.5 text-sm rounded-sm border text-black dark:text-white bg-neutral-50 dark:bg-neutral-900 border-neutral-300 dark:border-neutral-600 placeholder:italic placeholder:text-neutral-500 dark:placeholder:text-neutral-400"
      />
    </label>
    <button class="w-[2rem] text-action-" @click="performSearch">
      <SearchIcon class="w-full text-neutral-500" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { SearchIcon } from "@heroicons/vue/solid";
import { ref } from "vue";

let search = ref<string>("");

withDefaults(
  defineProps<{
    placeholder?: string;
  }>(),
  {
    placeholder: "search",
  },
);

const performSearch = () => console.log("perform search with...", search.value);
</script>
