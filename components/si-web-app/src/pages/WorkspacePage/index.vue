<template>
  <div id="app-main-layout" class="flex flex-col w-screen h-screen">
    <div
      id="workspace-view"
      class="flex flex-row w-full h-full overflow-hidden"
      v-else
    >
      <WorkspaceNav
        :organizationId="organizationId"
        :workspaceId="workspaceId"
      />

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
  mounted: function () {
    // TODO: Get the workspace and organization loaded into Vuex; back into the system.
    //
    //let user = this.$store.getters["user"];
    // Navigate to the default page: applications
    let userProfile = this.$store.getters["user/profile"];
    let workspace = this.$store.getters["workspace/current"];
    let url = `/o/:${userProfile.organization.id}/w/:${workspace.id}/a`;
    if (this.$route.path !== url) this.$router.push(url);
  },
};
</script>
