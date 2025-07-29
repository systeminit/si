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
    icon="settings"
    label="Manage Workspace Tokens"
    @select="openWorkspaceApiTokensHandler"
  />
  <DropdownMenuItem
    icon="clipboard-copy"
    label="Copy Workspace Token"
    @select="copyWorkspaceToken"
  />
  <DropdownMenuItem
    icon="settings"
    label="Workspace Integrations"
    @select="emit('openIntegrationsModal')"
  />
</template>

<script setup lang="ts">
import { DropdownMenuItem } from "@si/vue-lib/design-system";
import { computed, inject } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { assertIsDefined, Context } from "../types";

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const workspacesStore = useWorkspacesStore();

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;

const openWorkspaceDetailsHandler = () => {
  if (!ctx.workspacePk.value) return;
  window.open(
    `${AUTH_PORTAL_URL}/workspace/${ctx.workspacePk.value}`,
    "_blank",
  );
};

const openWorkspaceApiTokensHandler = () => {
  if (!ctx.workspacePk.value) return;
  window.open(
    `${AUTH_PORTAL_URL}/workspace/${ctx.workspacePk.value}/tokens`,
    "_blank",
  );
};

const openManageWorkspacesHandler = () => {
  if (!ctx.workspacePk.value) return;
  window.open(`${AUTH_PORTAL_URL}/workspaces/`, "_blank");
};

const workspaceToken = computed(() => workspacesStore.getWorkspaceToken);

const copyWorkspaceToken = () => {
  const token = workspaceToken.value;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  navigator.clipboard.writeText(token || "");
};

const emit = defineEmits([
  "openExportModal",
  "openImportModal",
  "openIntegrationsModal",
]);
</script>
