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
import { WorkspaceService } from "@/service/workspace";
import { onMounted } from "vue";

// FIXME(nick,adam,victor): this is a hack until applications are removed, the changeset button comes back, and deployment nodes no longer exist.
onMounted(WorkspaceService.setupWorkspaceWithDefaults);
</script>
