<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <NavbarButton icon="settings" tooltipText="Workspace Settings">
    <template #dropdownContent>
      <WorkspaceSettingsMenuItems
        @openImportModal="importModalRef?.open()"
        @openExportModal="exportModalRef?.open()"
        @openIntegrationsModal="integrationsModalRef?.open()"
      />
    </template>
    <WorkspaceImportModal ref="importModalRef" />
    <WorkspaceExportModal ref="exportModalRef" />
    <WorkspaceIntegrationsModal ref="integrationsModalRef" />
  </NavbarButton>
</template>

<script setup lang="ts">
import { ref } from "vue";
import WorkspaceImportModal from "@/components/WorkspaceImportModal.vue";
import WorkspaceExportModal from "@/components/WorkspaceExportModal.vue";
import WorkspaceIntegrationsModal from "@/components/WorkspaceIntegrationsModal.vue";
import WorkspaceSettingsMenuItems from "./WorkspaceSettingsMenuItems.vue";
import NavbarButton from "./NavbarButton.vue";

const importModalRef = ref<InstanceType<typeof WorkspaceImportModal>>();
const exportModalRef = ref<InstanceType<typeof WorkspaceExportModal>>();
const integrationsModalRef = ref<InstanceType<typeof WorkspaceIntegrationsModal>>();
</script>
