<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel
    rememberSizeKey="changeset-and-asset"
    side="left"
    :minSize="250"
  >
    <div class="flex flex-col h-full">
      <ChangeSetPanel ref="changeSetPanelRef" />
    </div>
  </ResizablePanel>

  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <GenericDiagram :customConfig="{}" :nodes="[]" :edges="[]" readOnly />
  </div>

  <ResizablePanel
    rememberSizeKey="component-details"
    side="right"
    :defaultSize="380"
    :minSize="300"
  >
  </ResizablePanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ResizablePanel } from "@si/vue-lib/design-system";
import ChangeSetPanel from "@/components/ChangeSetPanel.vue";
import GenericDiagram from "@/components/GenericDiagram/GenericDiagram.vue";
</script>
