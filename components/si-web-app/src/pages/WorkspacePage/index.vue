<template>
  <div id="app-main-layout" class="flex flex-col h-screen w-screen">
    <AppBar />

    <div
      id="workspace-view"
      class="flex flex-row h-full w-full overflow-hidden"
    >
      <WorkspaceNav
        :organizationId="organizationId"
        :workspaceId="workspaceId"
      />

      <div class="flex flex-col h-full w-full">
        <router-view class="w-full h-full overflow-auto" />
      </div>
    </div>
  </div>
</template>

<script>
import WorkspaceNav from "./WorkspaceNav.vue";
import AppBar from "@/components/common/AppBar.vue";

export default {
  name: "WorkspacePage",
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
  },
  components: {
    AppBar,
    WorkspaceNav,
  },
};
</script>