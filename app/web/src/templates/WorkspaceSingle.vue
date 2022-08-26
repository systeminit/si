<template>
  <div
    id="workspace"
    class="overflow-hidden flex flex-col w-full h-screen select-none"
  >
    <Navbar />

    <router-view class="overflow-hidden" />

    <StatusBar class="flex-none" />
    <SiToast />
  </div>
</template>

<script lang="ts" setup>
import { onMounted } from "vue";
import Navbar from "@/organisms/Navbar.vue";
import StatusBar from "@/organisms/StatusBar.vue";
import { WorkspaceService } from "@/service/workspace";
import SiToast from "@/atoms/SiToast.vue";

// FIXME(nick,adam,victor): this is a hack until applications are removed, the changeset button comes back, and deployment nodes no longer exist.
onMounted(WorkspaceService.setupWorkspaceWithDefaults);
</script>
