<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="changeset-and-asset" side="left" :min-size="250">
    <div class="flex flex-col h-full">
      <ChangeSetPanel ref="changeSetPanelRef" />
    </div>
  </SiPanel>

  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <GenericDiagram :custom-config="{}" :nodes="[]" :edges="[]" read-only />
  </div>

  <SiPanel
    remember-size-key="component-details"
    side="right"
    :default-size="380"
    :min-size="300"
  >
  </SiPanel>
</template>

<script lang="ts" setup>
import _ from "lodash";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import GenericDiagram from "@/organisms/GenericDiagram/GenericDiagram.vue";
</script>
