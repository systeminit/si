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
        <div class="flex w-full h-8 changeset-status-bar justify-between">
          <div class="flex">
            <div class="text-sm font-light pt-1 pl-6 text-gray-400">
              ChangeSet:
            </div>
            <div class="text-sm font-normal pt-1 pl-1 text-gray-400">
              alex/my-glorious-changeset
            </div>
          </div>

          <div class="flex">
            <div class="text-sm font-light pt-1 text-gray-400">status:</div>
            <div class="text-sm font-normal pt-1 pl-1 pr-6 text-orange-300">
              edit mode
            </div>
          </div>
        </div>

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

<style type="text/css" scoped>
.changeset-status-bar {
  background-color: #212324;
}
</style>
