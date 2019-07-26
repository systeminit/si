<template>
  <StandardLayout>
    <WorkspaceTop />
  </StandardLayout>
</template>

<script lang="ts">
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";
import WorkspaceTop from "@/components/WorkspaceTop.vue";

export default Vue.extend({
  name: "Workspaces",
  components: {
    StandardLayout,
    WorkspaceTop,
  },
});
</script>
