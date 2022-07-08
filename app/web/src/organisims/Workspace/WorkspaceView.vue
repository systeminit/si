<template>
  <Viewer
    :schematic-viewer-id="schematicViewerId"
    :viewer-state="viewerState"
    :viewer-event$="viewerEventObservable.viewerEvent$"
    :schematic-data="schematicData"
    :editor-context="editorContext"
    :schematic-kind="schematicKind"
    :deployment-node-selected="deploymentNodeSelected"
    :light-mode="true"
  />
  <SiSidebar :placeholder="'Component Panel'" />

  <!-- NOTE(nick): we will likely need to use a large z index to ensure that the profile dropdown is displayed -->
  <!-- The property panel should only be displayed when working with a node/component/object -->
  <!-- <SiSidebar
    :place-right="true"
    :placeholder="'Property Panel'"
    class="z-900"
  /> -->
  <StatusBar />
</template>

<script setup lang="ts">
import { SchematicKind } from "@/api/sdf/dal/schematic";
import Viewer from "@/organisims/SchematicViewer/Viewer.vue";
import * as VE from "@/organisims/SchematicViewer/viewer_event";
import _ from "lodash";
import { ViewerStateMachine } from "@/organisims/SchematicViewer/state_machine";
import StatusBar from "@/organisims/StatusBar.vue";
import SiSidebar from "@/molecules/SiSidebar.vue";

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
