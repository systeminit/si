<template>
  <AppLayout>
    <div class="text-white">
      <div
        v-if="workspacesReqStatus.isPending"
        class="w-full flex flex-col items-center gap-4 p-xl"
      >
        <Icon name="loader" size="2xl" />
        <h2>Loading your workspace(s)...</h2>
      </div>
      <ErrorMessage
        v-else-if="workspacesReqStatus.isError"
        :request-status="workspacesReqStatus"
      />
      <div
        v-else-if="workspacesReqStatus.isSuccess"
        class="w-full flex flex-col items-center gap-4 p-xl"
      >
        Finished loading!
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { useRouter } from "vue-router";
import { useWorkspacesStore } from "@/store/workspaces.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import AppLayout from "@/components/layout/AppLayout.vue";

const router = useRouter();

const workspacesStore = useWorkspacesStore();
const workspaces = computed(() => workspacesStore.allWorkspaces);

const workspacesReqStatus = workspacesStore.getRequestStatus(
  "FETCH_USER_WORKSPACES",
);

function autoSelectWorkspace() {
  if (workspaces.value.length !== 1) return;

  router.push({
    name: "workspace-single",
    params: { workspacePk: workspaces.value[0].pk },
  });
}

watch(
  workspacesReqStatus,
  () => {
    if (workspacesReqStatus.value.isSuccess) autoSelectWorkspace();
  },
  { immediate: true },
);
</script>
