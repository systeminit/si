<template>
  <WorkspacePage />
</template>

<script lang="ts">
import Vue from "vue";
import WorkspacePage from "@/pages/WorkspacePage/index.vue";

export default Vue.extend({
  name: "home",
  components: {
    WorkspacePage,
  },
});
</script>
