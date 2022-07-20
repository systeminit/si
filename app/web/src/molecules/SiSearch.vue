<template v-slot="search">
  <div class="flex nowrap p-4">
    <input v-model="search" placeholder="search" class="text-black" />
    <button @click="resetSearch">
      <XCircleIcon class="w-full" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { XCircleIcon } from "@heroicons/vue/outline";
import { ref } from "vue";

let search = ref<string>("");
const resetSearch = () => (search.value = "");
</script>
