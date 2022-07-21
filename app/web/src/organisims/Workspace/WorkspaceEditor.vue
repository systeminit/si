<template>
  <div class="w-full h-full flex pointer-events-none relative">
    <Viewer
      v-if="lightmode"
      :schematic-viewer-id="schematicViewerId"
      :viewer-state="viewerState"
      :viewer-event$="viewerEventObservable.viewerEvent$"
      :schematic-data="schematicData"
      :editor-context="editorContext"
      :schematic-kind="schematicKind"
      :deployment-node-selected="deploymentNodeSelected"
      :light-mode="true"
      class="pointer-events-auto"
    />
    <Viewer
      v-else
      :schematic-viewer-id="schematicViewerId"
      :viewer-state="viewerState"
      :viewer-event$="viewerEventObservable.viewerEvent$"
      :schematic-data="schematicData"
      :editor-context="editorContext"
      :schematic-kind="schematicKind"
      :deployment-node-selected="deploymentNodeSelected"
      :light-mode="false"
      class="pointer-events-auto"
    />

    <div class="flex flex-row w-full bg-transparent">
      <SiSidebar side="left" class="pointer-events-auto dark:text-white">
        <SiChangesetForm />
        <AssetsTabs />
      </SiSidebar>
      <!-- transparent div that flows through to the canvas -->
      <div class="grow h-full pointer-events-none"></div>
      <SiSidebar side="right" class="pointer-events-auto dark:text-white">
        <SiChangesetForm />
        <AssetsTabs />
      </SiSidebar>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SchematicKind } from "@/api/sdf/dal/schematic";
import Viewer from "@/organisims/SchematicViewer/Viewer.vue";
import * as VE from "@/organisims/SchematicViewer/viewer_event";
import _ from "lodash";
import { ViewerStateMachine } from "@/organisims/SchematicViewer/state_machine";
import SiSidebar from "@/atoms/SiSidebar.vue";
import { ThemeService } from "@/service/theme";
import { refFrom } from "vuse-rx/src";
import { computed } from "vue";
import { Theme } from "@/observable/theme";
import SiChangesetForm from "@/organisims/SiChangesetForm.vue";
import AssetsTabs from "@/organisims/AssetsTabs.vue";

const props = defineProps<{
  mutable: boolean;
}>();

const canoe = computed(() => {
  if (props.mutable) {
    return "compose canoe";
  }
  return "view canoe";
});

const schematicViewerId = _.uniqueId();
const viewerState = new ViewerStateMachine();
const viewerEventObservable = new VE.ViewerEventObservable();
const schematicData = {
  nodes: [],
  connections: [],
};
const editorContext = null;
const schematicKind = SchematicKind.Component;
const deploymentNodeSelected = null;

const theme = refFrom<Theme>(ThemeService.currentTheme());
const lightmode = computed(() => {
  return theme.value?.value == "light";
});
</script>
