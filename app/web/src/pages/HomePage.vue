<template>
  <AppLayout>
    <div class="text-white">
      <template v-if="workspacesReqStatus.isPending">
        <Icon name="loader" size="xl" />
        <h2>Loading your workspaces...</h2>
      </template>
      <ErrorMessage
        v-else-if="workspacesReqStatus.isError"
        :request-status="workspacesReqStatus"
      />
      <template v-else-if="workspacesReqStatus.isSuccess">
        Your workspaces...
      </template>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { useRouter } from "vue-router";
import { useWorkspacesStore } from "@/store/workspaces.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import AppLayout from "@/layout/AppLayout.vue";

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
    params: { workspaceId: workspaces.value[0].id },
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
