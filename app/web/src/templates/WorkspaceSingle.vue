<template>
  <div
    id="workspace"
    class="overflow-hidden flex flex-col w-full h-full select-none bg-white dark:bg-#[333333]"
  >
    <Navbar />

    <router-view />

    <div class="pointer-events-auto absolute bottom-0 z-10 w-screen">
      <StatusBar />
    </div>
  </div>
</template>

<script setup lang="ts">
import Navbar from "@/organisims/Navbar.vue";
import StatusBar from "@/organisims/StatusBar.vue";
import { onMounted } from "vue";
import { Viewer } from "@/observable/viewer";
import { ViewerService } from "@/service/viewer";
import { refFrom } from "vuse-rx";

const currentViewer = refFrom<Viewer>(ViewerService.currentViewer());

onMounted(() => {
  const currentViewerOnMount = currentViewer.value;
  if (currentViewerOnMount) {
    ViewerService.setTo(currentViewerOnMount);
  } else {
    ViewerService.setTo("compose");
  }
});
</script>
