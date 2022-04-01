<template>
  <div :id="viewerId" class="w-full h-full">
    <Viewer
      v-if="schematicData && editorContext && schematicKind"
      :schematic-viewer-id="viewerId"
      :viewer-state="viewerState"
      :schematic-data="schematicData"
      :viewer-event$="props.viewerEvent$"
      :editor-context="editorContext"
      :schematic-kind="schematicKind"
      :is-component-panel-pinned="isComponentPanelPinned"
      :component-panel-pin="componentPanelPin"
    />
  </div>
</template>

<script setup lang="ts">
import _ from "lodash";
import * as Rx from "rxjs";

import Viewer from "./SchematicViewer/Viewer.vue";

import { ViewerStateMachine } from "./SchematicViewer/state";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";

import { Schematic } from "./SchematicViewer/model";
import { refFrom } from "vuse-rx";
import { editSessionWritten$ } from "@/observable/edit_session";
import { applicationNodeId$ } from "@/observable/application";
import { system$ } from "@/observable/system";
import { visibility$ } from "@/observable/visibility";
import { EditorContext, SchematicKind } from "@/api/sdf/dal/schematic";
import { combineLatest, from } from "rxjs";
import { switchMap } from "rxjs/operators";
import { ViewerEvent } from "./SchematicViewer/event";
import { computed } from "vue";

const props = defineProps<{
  viewerEvent$: Rx.ReplaySubject<ViewerEvent | null> | undefined;
  schematicKind: SchematicKind | null;
  isComponentPanelPinned: boolean;
  deploymentComponentPin?: number;
}>();

const componentName = "SchematicViewer";
const componentId = _.uniqueId();

const viewerId = componentName + "-" + componentId;
const viewerState = new ViewerStateMachine();

const schematicData = refFrom<Schematic | null>(
  combineLatest([
    system$,
    applicationNodeId$,
    editSessionWritten$,
    visibility$,
  ]).pipe(
    switchMap(([system, applicationNodeId]) => {
      if (system && applicationNodeId) {
        return SchematicService.getSchematic({
          systemId: system.id,
          rootNodeId: applicationNodeId,
        });
      } else {
        return from([null]);
      }
    }),
    switchMap((schematic) => {
      if (schematic) {
        if (schematic.error) {
          GlobalErrorService.set(schematic);
          return from([null]);
        } else {
          return from([schematic]);
        }
      } else {
        return from([null]);
      }
    }),
  ),
);

const componentPanelPin = computed(() => {
  if (!schematicData.value || !props.deploymentComponentPin) return undefined;

  for (const node of schematicData.value.nodes) {
    if (node.kind.componentId === props.deploymentComponentPin) {
      return node.id;
    }
  }
  throw new Error(`Node wasn't found ${props.deploymentComponentPin}`);
});

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
