<template>
  <div class="w-full h-full flex relative overflow-hidden">
    <!-- TODO(victor): SiCanvas should be readonly on this mode. It's not right now -->
    <SiCanvas
      v-if="lightMode && editorContext && deploymentNode && schematicData"
      :deployment-node-selected="deploymentNode.id"
      :editor-context="editorContext"
      :schematic-data="schematicData"
      :schematic-kind="SchematicKind.Component"
      :schematic-viewer-id="schematicViewerId"
      :viewer-event$="viewerEventObservable.viewerEvent$"
      :viewer-state="viewerState"
      class="pointer-events-auto absolute z-10"
      light-mode
    />
    <SiCanvas
      v-else-if="editorContext && deploymentNode && schematicData"
      :deployment-node-selected="deploymentNode.id"
      :editor-context="editorContext"
      :schematic-data="schematicData"
      :schematic-kind="SchematicKind.Component"
      :schematic-viewer-id="schematicViewerId"
      :viewer-event$="viewerEventObservable.viewerEvent$"
      :viewer-state="viewerState"
      class="pointer-events-auto absolute z-10"
    />

    <div class="flex flex-row w-full bg-transparent">
      <SiSidebar side="left">
        <WorkspaceViewTabs
          :viewer-event$="viewerEventObservable.viewerEvent$"
        />
      </SiSidebar>

      <!-- transparent div that flows through to the canvas -->
      <div class="grow h-full pointer-events-none"></div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import SiCanvas from "@/organisms/SiCanvas.vue";
import {
  EditorContext,
  Schematic,
  SchematicKind,
  SchematicNode,
  SchematicSchemaVariants,
} from "@/api/sdf/dal/schematic";
import _ from "lodash";
import { refFrom, untilUnmounted } from "vuse-rx";
import { combineLatest, forkJoin, map, switchMap, take, tap } from "rxjs";
import { system$ } from "@/observable/system";
import { applicationNodeId$ } from "@/observable/application";
import {
  standardVisibilityTriggers$,
  visibility$,
} from "@/observable/visibility";
import { ThemeService } from "@/service/theme";
import { ref } from "vue";
import {
  schematicData$,
  schematicSchemaVariants$,
} from "@/observable/schematic";
import { ViewerStateMachine } from "@/organisms/SiCanvas/state_machine/machine";
import * as VE from "@/organisms/SiCanvas/viewer_event";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import SiSidebar from "@/atoms/SiSidebar.vue";
import WorkspaceViewTabs from "@/organisms/WorkspaceViewTabs.vue";

const schematicViewerId = _.uniqueId();
const viewerState = new ViewerStateMachine();
const viewerEventObservable = new VE.ViewerEventObservable();

// SiCanvas needs both schematic and schemaVariants to be loaded in before rendering
// TODO(victor): either move dependency loading to the canvas component or load them globally.
let oldSchematic: Schematic | undefined;
let oldSchemaVariants: SchematicSchemaVariants | undefined;
combineLatest([system$, standardVisibilityTriggers$])
  .pipe(
    untilUnmounted,
    map(([system]) =>
      system?.id !== undefined ? { systemId: system.id } : {},
    ),
    switchMap((systemRequest) =>
      forkJoin([
        SchematicService.getSchematic(systemRequest).pipe(take(1)),
        SchematicService.listSchemaVariants().pipe(take(1)),
      ]),
    ),
  )
  .subscribe(([schematic, variants]) => {
    if (schematic.error) {
      GlobalErrorService.set(schematic);
      return;
    }

    if (!oldSchematic || !_.isEqual(oldSchematic, schematic)) {
      oldSchematic = schematic as Schematic;
      schematicData$.next(schematic as Schematic);
    }

    if (!oldSchemaVariants || !_.isEqual(oldSchemaVariants, variants)) {
      oldSchemaVariants = variants as SchematicSchemaVariants;
      schematicSchemaVariants$.next(variants as SchematicSchemaVariants);
    }
  });

const editorContext = refFrom<EditorContext | null>(
  combineLatest([system$, applicationNodeId$, visibility$]).pipe(
    map(([system, applicationNodeId]) =>
      applicationNodeId ? { systemId: system?.id, applicationNodeId } : null,
    ),
  ),
);

// We're only showing a components canvas, which in our current architecture is linked to a deployment node, so we need to load the default one
const deploymentNode = ref<SchematicNode | null>(null);
const schematicData = refFrom<Schematic>(
  schematicData$.pipe(
    tap((sd) => {
      if (!sd) {
        deploymentNode.value = null;
        return;
      }
      for (const node of sd.nodes) {
        if (node.kind.kind == "deployment") {
          deploymentNode.value = node;
          break;
        }
      }
    }),
    map((sd) => sd ?? { nodes: [], connections: [] }),
  ),
);

const lightMode = refFrom<boolean>(
  ThemeService.currentTheme().pipe(map((theme) => theme?.value == "light")),
);
</script>
