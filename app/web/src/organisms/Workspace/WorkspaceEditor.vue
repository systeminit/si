<template>
  <div class="w-full h-full flex pointer-events-none relative overflow-hidden">
    <!-- FIXME(nick,victor): remove reliance on z index -->
    <SiCanvas
      v-if="lightmode && editorContext"
      light-mode
      :schematic-viewer-id="schematicViewerId"
      :viewer-state="viewerState"
      :editor-context="editorContext"
      :schematic-data="schematicData"
      :viewer-event$="viewerEventObservable.viewerEvent$"
      :schematic-kind="SchematicKind.Deployment"
      :deployment-node-selected="null"
      class="pointer-events-auto absolute z-10"
    />
    <SiCanvas
      v-else-if="editorContext"
      :schematic-viewer-id="schematicViewerId"
      :viewer-state="viewerState"
      :editor-context="editorContext"
      :schematic-data="schematicData"
      :viewer-event$="viewerEventObservable.viewerEvent$"
      :schematic-kind="SchematicKind.Deployment"
      :deployment-node-selected="null"
      class="pointer-events-auto absolute z-10"
    />

    <div class="flex flex-row w-full bg-transparent">
      <SiSidebar side="left">
        <SiChangesetForm />
        <AssetsTabs :viewer-event$="viewerEventObservable.viewerEvent$" />
      </SiSidebar>

      <!-- transparent div that flows through to the canvas -->
      <div class="grow h-full pointer-events-none"></div>

      <SiSidebar side="right" :hidden="activeNode === null">
        <ComponentDetails />
      </SiSidebar>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  EditorContext,
  Schematic,
  SchematicKind,
  SchematicSchemaVariants,
} from "@/api/sdf/dal/schematic";
import SiCanvas from "@/organisms/SiCanvas.vue";
import * as VE from "@/organisms/SiCanvas/viewer_event";
import _ from "lodash";
import { ViewerStateMachine } from "@/organisms/SiCanvas/state_machine";
import SiSidebar from "@/atoms/SiSidebar.vue";
import { ThemeService } from "@/service/theme";
import { refFrom, untilUnmounted } from "vuse-rx";
import { computed, ref } from "vue";
import { Theme } from "@/observable/theme";
import SiChangesetForm from "@/organisms/SiChangesetForm.vue";
import { combineLatest, forkJoin, from, map, switchMap, take } from "rxjs";
import { GetSchematicArgs } from "@/service/schematic/get_schematic";
import {
  standardVisibilityTriggers$,
  visibility$,
} from "@/observable/visibility";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import {
  schematicData$,
  schematicSchemaVariants$,
} from "@/observable/schematic";
import { system$ } from "@/observable/system";
import { applicationNodeId$ } from "@/observable/application";
import AssetsTabs from "@/organisms/AssetsTabs.vue";
import { lastSelectedNode$ } from "@/observable/selection";
import ComponentDetails from "@/organisms/ComponentDetails.vue";

defineProps<{
  mutable: boolean;
}>();

const schematicViewerId = _.uniqueId();
const viewerState = new ViewerStateMachine();
const viewerEventObservable = new VE.ViewerEventObservable();
const schematicData = ref<Schematic>({ nodes: [], connections: [] });

schematicData$.subscribe((schematic) => {
  if (schematic) {
    schematicData.value = schematic;
  } else {
    schematicData.value = { nodes: [], connections: [] };
  }
});

let oldSchematic: Schematic | undefined;
combineLatest([
  system$.pipe(
    map((system) => {
      const request: GetSchematicArgs = {};
      if (system) {
        request.systemId = system.id;
      }
      return request;
    }),
  ),
  standardVisibilityTriggers$,
])
  .pipe(
    switchMap(([request]) => {
      const variants = SchematicService.listSchemaVariants().pipe(take(1));
      const schematic = SchematicService.getSchematic(request).pipe(take(1));
      return from([[variants, schematic]]);
    }),
    switchMap((calls) => {
      return forkJoin(calls);
    }),
  )
  .pipe(untilUnmounted)
  .subscribe(([variants, schematic]) => {
    if (variants.error) {
      GlobalErrorService.set(variants);
      return from([]);
    }

    if (schematic.error) {
      GlobalErrorService.set(schematic);
      return from([]);
    }

    // If the schematic didn't change, but standard visibility triggers forced a refetch, we have to ignore it
    // We avoid passing this stale data around (it races making nodes teleport right after a move, as the local data is more up to date)
    if (!oldSchematic || !_.isEqual(oldSchematic, schematic)) {
      oldSchematic = schematic as Schematic;
      schematicData$.next(schematic as Schematic);
      schematicSchemaVariants$.next(variants as SchematicSchemaVariants);
    }
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

const activeNode = refFrom(lastSelectedNode$);

const theme = refFrom<Theme>(ThemeService.currentTheme());
const lightmode = computed(() => {
  return theme.value?.value == "light";
});
</script>
