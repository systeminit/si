<template>
  <div id="app-main-layout" class="flex flex-col w-screen h-screen">
    <div
      id="workspace-view"
      class="flex flex-row w-full h-full overflow-hidden"
    >
      <WorkspaceNav />

      <div class="flex flex-col w-full h-full">
        <router-view class="w-full h-full overflow-auto" />
      </div>
    </div>
  </div>
</template>

<script>
import { mapState } from "vuex";

import WorkspaceNav from "./WorkspaceNav.vue";

export default {
  name: "WorkspacePage",
  components: {
    WorkspaceNav,
  },
  async mounted() {
    let organization = this.$store.getters["organization/current"];
    let workspace = this.$store.getters["workspace/current"];
    let url = `/o/:${organization.id}/w/:${workspace.id}/a`;
    if (this.$route.path !== url) this.$router.push(url);
  },
};
</script>
