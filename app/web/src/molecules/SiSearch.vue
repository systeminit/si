<template v-slot="search">
  <input v-model="search" placeholder="search" class="text-black" />
  <button @click="resetSearch">X</button>
</template>

<script setup lang="ts">
import { ref } from "vue";

let search = ref<string>("");
const resetSearch = () => (search.value = "");
</script>
