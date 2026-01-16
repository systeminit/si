<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <DropdownMenuItem icon="settings" label="Manage Workspaces" @select="openManageWorkspacesHandler" />
  <DropdownMenuItem icon="cloud-download" label="Import Workspace" @select="emit('openImportModal')" />
  <DropdownMenuItem icon="edit" label="Manage Users" @select="openWorkspaceDetailsHandler" />
  <DropdownMenuItem icon="settings" label="Manage Workspace Tokens" @select="openWorkspaceApiTokensHandler" />
  <DropdownMenuItem icon="clipboard-copy" label="Copy Workspace Token" @select="copyWorkspaceToken" />
  <DropdownMenuItem icon="settings" label="Workspace Integrations" @select="emit('openIntegrationsModal')" />
</template>

<script setup lang="ts">
import { DropdownMenuItem } from "@si/vue-lib/design-system";
import { useWorkspacesStore } from "@/store/workspaces.store";

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const workspacesStore = useWorkspacesStore();

const openWorkspaceDetailsHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspace/${currentWorkspace}`, "_blank");
};

const openWorkspaceApiTokensHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspace/${currentWorkspace}/tokens`, "_blank");
};

const openManageWorkspacesHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspaces/`, "_blank");
};

const copyWorkspaceToken = () => {
  const currentWorkspace = workspacesStore.selectedWorkspace;

  navigator.clipboard.writeText(currentWorkspace?.token || "");
};

const emit = defineEmits(["openExportModal", "openImportModal", "openIntegrationsModal"]);
</script>
