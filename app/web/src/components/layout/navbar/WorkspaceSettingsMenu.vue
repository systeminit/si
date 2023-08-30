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
import NavbarButton from "./NavbarButton.vue";

const importModalRef = ref<InstanceType<typeof WorkspaceImportModal>>();
const exportModalRef = ref<InstanceType<typeof WorkspaceExportModal>>();
</script>
