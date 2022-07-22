<template>
  <div class="w-full h-full flex pointer-events-none relative">
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

      <SiSidebar side="right">
        <div class="text-center mt-10">
          poop canoe
          <div v-if="props.mutable">(rw)</div>
          <div v-else>(ro)</div>
        </div>
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
import SiCanvas from "@/organisms/SchematicViewer/SiCanvas.vue";
import * as VE from "@/organisms/SchematicViewer/viewer_event";
import _ from "lodash";
import { ViewerStateMachine } from "@/organisms/SchematicViewer/state_machine";
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
import { ChangeSetService } from "@/service/change_set";
import { ApplicationService } from "@/service/application";
import AssetsTabs from "@/organisms/AssetsTabs.vue";

const props = defineProps<{
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

// FIXME(nick,adam): create an application and a changeset when the editor is loaded.
// We need both in order to create and drag nodes.

ApplicationService.currentApplication().subscribe((application) => {
  if (application === null) {
    ApplicationService.createApplication({
      name: "poop",
    }).subscribe((response) => {
      if (response.error) {
        console.log("oopsie poopsie! we could not create an application!");
        GlobalErrorService.set(response);
        return;
      }

      ApplicationService.clearCurrentApplication();
      ApplicationService.setCurrentApplication({
        applicationId: response.application.id,
      }).subscribe((response) => {
        if (response.error) {
          console.log("could not set current application to poop!");
          GlobalErrorService.set(response);
          return;
        }
      });
    });
  }
});

ChangeSetService.currentChangeSet().subscribe((changeSet) => {
  if (changeSet === null) {
    ChangeSetService.createChangeSet({ changeSetName: "canoe" }).subscribe(
      (response) => {
        if (response.error) {
          console.log("oopsie poopsie! we could not create a change set!");
          GlobalErrorService.set(response);
          return;
        }

        ChangeSetService.startEditSession({
          changeSetPk: response.changeSet.pk,
        }).subscribe((response) => {
          if (response.error) {
            console.log("could not start edit session!");
            GlobalErrorService.set(response);
            return;
          }
        });
      },
    );
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

const theme = refFrom<Theme>(ThemeService.currentTheme());
const lightmode = computed(() => {
  return theme.value?.value == "light";
});
</script>
