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
import * as _ from "lodash-es";
import ChangeSetPanel from "@/components/ChangeSetPanel.vue";
import GenericDiagram from "@/components/GenericDiagram/GenericDiagram.vue";
import SiPanel from "@/components/SiPanel.vue";
</script>
