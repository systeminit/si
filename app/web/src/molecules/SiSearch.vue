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
        class="w-full text-black p-1 pl-6 text-xs rounded-sm border dark:text-black bg-neutral-100 border-neutral-200 placeholder:italic placeholder:text-neutral-400"
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

const resetSearch = () => (search.value = "");

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
