<template>
  <div class="w-full h-full flex pointer-events-none">
    <div class="w-full h-full z-0 relative">
      <Viewer
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
      <div class="absolute inset-0 z-10">
        <div class="flex flex-col h-full">
          <!-- panels -->
          <div class="flex flex-row grow">
            <SiSidebar side="left" class="pointer-events-auto dark:text-white"
              >poop</SiSidebar
            >
            <!-- transparent div that flows through to the canvas -->
            <div class="grow bg-transparent h-full pointer-events-none"></div>
            <SiSidebar side="right" class="pointer-events-auto dark:text-white"
              >canoe</SiSidebar
            >
          </div>
          <!-- status bar -->
          <div class="pointer-events-auto">
            <StatusBar />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SchematicKind } from "@/api/sdf/dal/schematic";
import Viewer from "@/organisims/SchematicViewer/Viewer.vue";
import * as VE from "@/organisims/SchematicViewer/viewer_event";
import _ from "lodash";
import { ViewerStateMachine } from "@/organisims/SchematicViewer/state_machine";
import StatusBar from "@/organisims/StatusBar.vue";
import SiSidebar from "@/atoms/SiSidebar.vue";

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
</script>
