<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel rememberSizeKey="changeset-and-asset" side="left" :minSize="250">
    <div class="flex flex-col h-full">
      <ChangeSetPanel ref="changeSetPanelRef" />
    </div>
  </SiPanel>

  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <GenericDiagram :customConfig="{}" :nodes="[]" :edges="[]" readOnly />
  </div>

  <SiPanel
    rememberSizeKey="component-details"
    side="right"
    :defaultSize="380"
    :minSize="300"
  >
  </SiPanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import ChangeSetPanel from "@/components/ChangeSetPanel.vue";
import GenericDiagram from "@/components/GenericDiagram/GenericDiagram.vue";
import SiPanel from "@/components/SiPanel.vue";
</script>
