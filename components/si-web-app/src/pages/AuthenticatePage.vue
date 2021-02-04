<template>
  <div
    class="flex flex-col h-screen bg-black justify-center items-center text-white bg-gray-800"
  >
    <router-view />
  </div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "AuthenticatePage",
});
</script>
