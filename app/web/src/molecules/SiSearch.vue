<template v-slot="search">
  <div
    class="flex nowrap gap-3 p-3 dark:border-[#525252] border-b-2 justify-between"
  >
    <label
      class="relative text-[#A4A5A6] focus-within:text-gray-600 block flex-grow"
    >
      <SearchIcon class="w-3.5 absolute top-2 left-1.5" />
      <input
        v-model="search"
        placeholder="search"
        class="w-full text-black p-1 pl-6 text-xs rounded-sm border dark:text-black bg-[#F7F7F7] border-[#E9EAEB] placeholder:italic placeholder:text-[#A4A5A6]"
      />
    </label>
    <button class="w-5" @click="resetSearch">
      <XCircleIcon class="w-full" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { XCircleIcon, SearchIcon } from "@heroicons/vue/outline";
import { ref } from "vue";

let search = ref<string>("");
const resetSearch = () => (search.value = "");
</script>
