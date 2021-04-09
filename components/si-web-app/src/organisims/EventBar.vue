<template>
  <div class="flex items-center w-full h-6 pt-1 pb-1 event-bar"></div>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "EventBar",
});
</script>

<style>
.event-bar {
  background-color: #1f2324;
  border-top: 1px solid #2f2f2f;
}
</style>
