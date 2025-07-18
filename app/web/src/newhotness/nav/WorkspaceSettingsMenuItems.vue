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
import { inject } from "vue";
import { assertIsDefined, Context } from "../types";
import { tokensByWorkspacePk } from "../logic_composables/tokens";

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

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

const copyWorkspaceToken = () => {
  const token = tokensByWorkspacePk[ctx.workspacePk.value];
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  navigator.clipboard.writeText(token || "");
};

const emit = defineEmits([
  "openExportModal",
  "openImportModal",
  "openIntegrationsModal",
]);
</script>
