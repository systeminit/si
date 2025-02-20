<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <DropdownMenuItem
    icon="settings"
    label="Manage Workspaces"
    @select="openManageWorkspacesHandler"
  />
  <DropdownMenuItem
    icon="cloud-download"
    label="Import Workspace"
    @select="emit('openImportModal')"
  />
  <DropdownMenuItem
    icon="edit"
    label="Manage Users"
    @select="openWorkspaceDetailsHandler"
  />
  <DropdownMenuItem
    v-if="showViz"
    icon="diagram"
    label="Visualizer"
    @select="gotoViz"
  />
  <DropdownMenuItem
    icon="clipboard-copy"
    label="Copy Workspace Token"
    @select="copyWorkspaceToken"
  />
  <DropdownMenuItem
    v-if="featureFlagsStore.SLACK_WEBHOOK"
    icon="settings"
    label="Workspace Integrations"
    @select="emit('openIntegrationsModal')"
  />
</template>

<script setup lang="ts">
import { DropdownMenuItem } from "@si/vue-lib/design-system";
import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const workspacesStore = useWorkspacesStore();
const changeSetStore = useChangeSetsStore();
const featureFlagsStore = useFeatureFlagsStore();
const router = useRouter();
const route = useRoute();

const openWorkspaceDetailsHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspace/${currentWorkspace}`, "_blank");
};

const openManageWorkspacesHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspaces/`, "_blank");
};

const showViz = computed(() => {
  return changeSetStore.selectedChangeSetId;
});

const gotoViz = () => {
  router.push({
    name: "workspace-viz",
    params: {
      ...route.params,
      changeSetId: changeSetStore.selectedChangeSetId,
    },
  });
};

const copyWorkspaceToken = () => {
  const currentWorkspace = workspacesStore.selectedWorkspace;
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  navigator.clipboard.writeText(currentWorkspace?.token || "");
};

const emit = defineEmits([
  "openExportModal",
  "openImportModal",
  "openIntegrationsModal",
]);
</script>
