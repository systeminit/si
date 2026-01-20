<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <NavbarButton icon="odin" size="sm">
    <template #dropdownContent>
      <SQLiteToolsMenuItems
        :changeSetId="changeSetId"
        :workspaceId="workspaceId"
        @hammer="hammerModalRef.open"
      />
    </template>
  </NavbarButton>
  <SQLiteToolsHammerModal 
    ref="hammerModalRef"
    :changeSetId="changeSetId"
    :workspaceId="workspaceId"
  />
</template>

<script setup lang="ts">
import NavbarButton from '@/components/layout/navbar/NavbarButton.vue';
import SQLiteToolsMenuItems from "./SQLiteToolsMenuItems.vue";
import SQLiteToolsHammerModal from './SQLiteToolsHammerModal.vue';
import { ref } from 'vue';

const props = defineProps<{
  changeSetId: string;
  workspaceId: string;
}>();

const hammerModalRef = ref();
</script>