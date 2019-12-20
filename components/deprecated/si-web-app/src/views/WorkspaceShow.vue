<template>
  <StandardLayout>
    <WorkspaceFull :workspace_id="this.$route.params.id" />
  </StandardLayout>
</template>

<script lang="ts">
import Vue from "vue";
import StandardLayout from "@/components/StandardLayout.vue";
import WorkspaceFull from "@/components/WorkspaceFull.vue";

export default Vue.extend({
  name: "WorkspaceShow",
  data() {
    return {
      workspace_id: "0",
    };
  },
  components: {
    StandardLayout,
    WorkspaceFull,
  },
  //beforeRouteUpdate(to, from, next) {
  //  console.log(to.params);
  //  this.workspace_id = to.params.id;
  //  next();
  //},
});
</script>
