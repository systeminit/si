<template>
  <div id="app-main-layout" class="flex flex-col h-screen w-screen">
    <AppBar />

    <div
      v-if="loading"
      class="flex flex-row h-full w-full bg-black text-white h-center"
    >
      <div class="object-center">
        Loading your workspace! Sit tight!
      </div>
    </div>
    <div
      id="workspace-view"
      class="flex flex-row h-full w-full overflow-hidden"
      v-else
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
  async created() {
    await this.$store.dispatch("loader/load");
  },
};
</script>
