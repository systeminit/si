<template>
  <div id="app-main-layout" class="flex flex-col h-screen w-screen">
    <AppBar />

    <div id="workspace-view" class="flex flex-row h-full w-full">
      <WorkspaceNav
        :organizationId="organizationId"
        :workspaceId="workspaceId"
      />

      <router-view class="w-full" />
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
