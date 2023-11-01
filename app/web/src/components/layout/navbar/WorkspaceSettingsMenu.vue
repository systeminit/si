<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <NavbarButton tooltipText="Workspace Settings">
    <Icon name="settings" />

    <template #dropdownContent>
      <DropdownMenuItem
        icon="cloud-upload"
        label="Export Workspace"
        @click="exportModalRef?.open()"
      />
      <DropdownMenuItem
        icon="cloud-download"
        label="Import Workspace"
        @click="importModalRef?.open()"
      />
      <DropdownMenuItem
        icon="edit"
        label="Manage Users"
        @click="openWorkspaceDetailsHandler"
      />
    </template>
  </NavbarButton>

  <WorkspaceImportModal ref="importModalRef" />
  <WorkspaceExportModal ref="exportModalRef" />
</template>

<script setup lang="ts">
import { DropdownMenuItem, Icon } from "@si/vue-lib/design-system";
import { ref } from "vue";
import WorkspaceImportModal from "@/components/WorkspaceImportModal.vue";
import WorkspaceExportModal from "@/components/WorkspaceExportModal.vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import NavbarButton from "./NavbarButton.vue";

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const importModalRef = ref<InstanceType<typeof WorkspaceImportModal>>();
const exportModalRef = ref<InstanceType<typeof WorkspaceExportModal>>();

const workspacesStore = useWorkspacesStore();

const openWorkspaceDetailsHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspace/${currentWorkspace}`, "_blank");
};
</script>
