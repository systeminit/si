<template>
  <div
    id="workspace"
    class="overflow-hidden flex flex-col w-full h-full select-none"
  >
    <Navbar />

    <router-view class="overflow-hidden" />

    <StatusBar class="flex-initial" />
  </div>
</template>

<script setup lang="ts">
import Navbar from "@/organisms/Navbar.vue";
import StatusBar from "@/organisms/StatusBar.vue";
import { onMounted } from "vue";
import _ from "lodash";
import { WorkspaceService } from "@/service/workspace";

onMounted(() => {
  // FIXME(nick,adam,victor): this is a hack until applications are removed, the changeset button comes back, and deployment nodes no longer exist.
  WorkspaceService.setupWorkspaceWithDefaults();
});
</script>
