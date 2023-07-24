<template>
  <div class="overflow-hidden">
    <RichText>
      <h3>Your dashboard</h3>
      <p>
        From here you can log into your local dev instance. Eventually this will
        be where you can manage multiple workspaces, users, organizations, etc.
      </p>
    </RichText>

    <template v-if="loadWorkspacesReqStatus.isPending">
      <Icon name="loader" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isError">
      <ErrorMessage :requestStatus="loadWorkspacesReqStatus" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isSuccess">
      <Stack class="mt-lg">
        <WorkspaceLinkWidget
          v-for="workspace in workspaces"
          :key="workspace.id"
          :workspaceId="workspace.id"
        />
      </Stack>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { Icon, RichText, Stack, ErrorMessage } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import WorkspaceLinkWidget from "@/components/WorkspaceLinkWidget.vue";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();

const workspaces = computed(() => workspacesStore.workspaces);

useHead({ title: "Dashboard" });

const loadWorkspacesReqStatus =
  workspacesStore.getRequestStatus("LOAD_WORKSPACES");

function reloadWorkspaces() {
  if (import.meta.env.SSR) return;
  if (!authStore.userIsLoggedIn) return;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
}

watch(() => authStore.userIsLoggedIn, reloadWorkspaces, { immediate: true });
</script>
