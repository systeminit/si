<template>
  <AppLayout>
    <div v-if="!workspace">
      <!-- WORKSPACE SELECTOR GOES HERE! -->
      <!-- TODO(wendy/theo) - Add workspace selector here. -->
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { firstValueFrom } from "rxjs";
import { refFrom } from "vuse-rx/src";
import { SessionService } from "@/service/session";
import { Workspace } from "@/api/sdf/dal/workspace";
import { WorkspaceService } from "@/service/workspace";
import AppLayout from "./AppLayout.vue";

const route = useRoute();
const router = useRouter();

const workspace = refFrom<Workspace | null>(
  WorkspaceService.currentWorkspace(),
);

onMounted(async () => {
  const defaults = await firstValueFrom(SessionService.getDefaults());
  if (route.name === "workspace-index" && !defaults.error && workspace.value) {
    await router.push({
      name: "workspace-single",
      path: "/w/:workspaceId",
      params: { workspaceId: workspace.value.id },
    });
  }
});
</script>
