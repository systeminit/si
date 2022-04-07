<template>
  <div :id="viewerId" class="w-full h-full">
    <Viewer
      v-if="showViewer"
      :schematic-viewer-id="viewerId"
      :viewer-state="viewerState"
      :editor-context="editorContext ?? null"
      :schematic-data="props.schematicData"
      :viewer-event$="props.viewerEvent$"
      :schematic-kind="props.schematicKind ?? null"
      :is-component-panel-pinned="props.isComponentPanelPinned"
      :deployment-node-pin="props.deploymentNodePin"
    />
  </div>
</template>

<script setup lang="ts">
import _ from "lodash";
import * as Rx from "rxjs";

import Viewer from "./SchematicViewer/Viewer.vue";

import { ViewerStateMachine } from "./SchematicViewer/state";

import { Schematic } from "./SchematicViewer/model";
import { refFrom } from "vuse-rx";
import { applicationNodeId$ } from "@/observable/application";
import { system$ } from "@/observable/system";
import { visibility$ } from "@/observable/visibility";
import { EditorContext, SchematicKind } from "@/api/sdf/dal/schematic";
import { combineLatest, from } from "rxjs";
import { switchMap } from "rxjs/operators";
import { ViewerEvent } from "./SchematicViewer/event";
import { computed } from "vue";

const showViewer = computed(() => {
  if (props.schematicData && editorContext.value && props.schematicKind) {
    // Component panels pointing to a null deployment will sync selection with deployment panel
    // To avoid this we don't render a component panel pointing to a invalid deployment
    const isComponent = props.schematicKind === SchematicKind.Component;
    if (isComponent && props.deploymentNodePin === undefined) {
      return false;
    }
    return true;
  }
  return false;
});

const props = defineProps<{
  viewerEvent$: Rx.ReplaySubject<ViewerEvent | null> | undefined;
  schematicKind: SchematicKind | null;
  isComponentPanelPinned: boolean;
  deploymentNodePin?: number;
  schematicData: Schematic | null;
}>();

const componentName = "SchematicViewer";
const componentId = _.uniqueId();

const viewerId = componentName + "-" + componentId;
const viewerState = new ViewerStateMachine();

const editorContext = refFrom<EditorContext | null>(
  combineLatest([system$, applicationNodeId$, visibility$]).pipe(
    switchMap(([system, applicationNodeId]) => {
      if (applicationNodeId) {
        return from([{ systemId: system?.id, applicationNodeId }]);
      } else {
        return from([null]);
      }
    }),
  ),
);
</script>
