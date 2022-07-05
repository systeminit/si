<template>
  <div class="flex flex-col items-center justify-center h-screen bg-white">
    <router-view class="flex" />

    <div class="flex m-6 border-2 p-2 rounded-lg border-black">
      <OldAppSwitch />
    </div>
  </div>
</template>

<script setup lang="ts">
import OldAppSwitch from "@/atoms/OldAppSwitch.vue";
import { computed, onMounted } from "vue";
import { SessionService } from "@/service/session";
import { useRouter, useRoute } from "vue-router";
import { firstValueFrom } from "rxjs";
import { Workspace } from "@/api/sdf/dal/workspace";
import { WorkspaceService } from "@/service/workspace";
import { refFrom } from "vuse-rx/src";

const route = useRoute();
const router = useRouter();

onMounted(async () => {
  const defaults = await firstValueFrom(SessionService.getDefaults());
  if (route.name == "new" && !defaults.error) {
    await router.push({
      name: "workspace-view",
      path: "/new/w/:workspaceId",
      params: { workspaceId: workspaceId.value },
    });
  }
});

const workspace = refFrom<Workspace | null>(
  WorkspaceService.currentWorkspace(),
);
const workspaceId = computed((): number | undefined => {
  if (workspace.value) {
    return workspace.value.id;
  }
  return undefined;
});
</script>
