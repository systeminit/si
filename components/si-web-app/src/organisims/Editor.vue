<template>
  <div class="flex flex-col h-full w-full">
    <PanelTree />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import PanelTree from "@/organisims/PanelTree.vue";

export default Vue.extend({
  name: "Editor",
  components: {
    PanelTree,
  },
});
</script>
