<template>
  <div class="overflow-hidden">
    <div
      class="pb-md flex flex-row gap-sm align-middle items-center justify-between"
    >
      <div>
        <div class="text-lg font-bold pb-sm">Your dashboard</div>
        <div v-if="featureFlagsStore.CREATE_WORKSPACES">
          From here you can log into any of your workspaces.
        </div>
        <div v-else>
          From here you can log into your local dev instance. Eventually this
          will be where you can manage multiple workspaces, users,
          organizations, etc.
        </div>
      </div>
      <VButton
        v-if="featureFlagsStore.CREATE_WORKSPACES"
        label="Create Workspace"
        icon="plus"
        :linkTo="{ name: 'workspace-settings', params: { workspaceId: 'new' } }"
      />
    </div>
    <template v-if="loadWorkspacesReqStatus.isPending">
      <Icon name="loader" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isError">
      <ErrorMessage :requestStatus="loadWorkspacesReqStatus" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isSuccess">
      <Stack>
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
import { Icon, Stack, ErrorMessage, VButton } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import WorkspaceLinkWidget from "@/components/WorkspaceLinkWidget.vue";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();
const featureFlagsStore = useFeatureFlagsStore();

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
