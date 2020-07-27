<template>
  <div id="app-main-layout" class="flex flex-col w-screen h-screen">
    <AppBar />

    <div
      v-if="loading"
      class="flex flex-row w-full h-full text-white bg-black h-center"
    >
      <div class="object-center">
        Loading your workspace! Sit tight!
      </div>
    </div>
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
  computed: {
    ...mapState({
      loading: state => state.loader.loading,
    }),
  },
};
</script>
