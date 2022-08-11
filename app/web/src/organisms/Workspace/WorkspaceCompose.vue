<template>
  <div class="w-full h-full flex pointer-events-none relative overflow-hidden">
    <!-- FIXME(nick,victor): remove reliance on z index -->
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
        <ChangeSetPanel class="border-b-2 dark:border-neutral-500 mb-2" />
        <AssetsTabs :viewer-event$="viewerEventObservable.viewerEvent$" />
      </SiSidebar>

      <!-- transparent div that flows through to the canvas -->
      <div class="grow h-full pointer-events-none"></div>

      <SiSidebar
        :hidden="
          activeNode === null || selectedComponentIdentification === null
        "
        side="right"
      >
        <ComponentDetails
          v-if="selectedComponentIdentification && selectedComponentName"
          :component-identification="selectedComponentIdentification"
          :component-name="selectedComponentName"
        />
      </SiSidebar>
    </div>
  </div>
</template>

<script lang="ts" setup>
import {
  EditorContext,
  Schematic,
  SchematicKind,
  SchematicNode,
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
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import {
  combineLatest,
  firstValueFrom,
  forkJoin,
  from,
  map,
  switchMap,
  take,
  tap,
} from "rxjs";
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
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { LabelList } from "@/api/sdf/dal/label_list";
import { ComponentService } from "@/service/component";
import { Node } from "@/organisms/SiCanvas/canvas/obj/node";

const schematicViewerId = _.uniqueId();
const viewerState = new ViewerStateMachine();
const viewerEventObservable = new VE.ViewerEventObservable();

const selectedComponentId = ref<number | "">("");
const activeNode = refFrom(lastSelectedNode$);

const selectedComponentName = computed(() => {
  if (selectedComponentId.value && componentIdentificationList.value) {
    for (const identification of componentIdentificationList.value) {
      if (
        identification.value &&
        identification.value.componentId === selectedComponentId.value
      ) {
        return identification.label;
      }
    }
  }
  return null;
});

const componentIdentificationList = refFrom<
  LabelList<ComponentIdentification | "">
>(
  ComponentService.listComponentsIdentification().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        const list: LabelList<ComponentIdentification | ""> = _.cloneDeep(
          response.list,
        );
        list.push({ label: "", value: "" });
        return from([list]);
      }
    }),
  ),
);

const componentRecord = computed(
  (): Record<number, ComponentIdentification> => {
    let record: Record<number, ComponentIdentification> = {};
    if (componentIdentificationList.value) {
      for (const item of componentIdentificationList.value) {
        if (item.value !== "") {
          record[item.value.componentId] = item.value;
        }
      }
    }
    return record;
  },
);

const selectedComponentIdentification = computed(
  (): ComponentIdentification | null => {
    if (selectedComponentId.value) {
      let record = componentRecord.value[selectedComponentId.value];
      if (record === null || record === undefined) {
        return null;
      }
      return componentRecord.value[selectedComponentId.value];
    }
    return null;
  },
);

const updateSelection = (node: Node | null) => {
  const componentId = node?.nodeKind?.componentId;

  // FIXME(nick): re-add locking for the view-only mode.
  // if (isPinned.value) return;

  // Ignores deselection and fake nodes, as they don't have any attributes
  if (!componentId || componentId === -1) return;

  selectedComponentId.value = componentId;
};
lastSelectedNode$
  .pipe(untilUnmounted)
  .subscribe((node) => updateSelection(node));
firstValueFrom(lastSelectedNode$).then((last) => updateSelection(last));

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

let oldSchematic: Schematic | undefined;
let oldSchemaVariants: SchematicSchemaVariants | undefined;
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

const lightMode = refFrom<boolean>(
  ThemeService.currentTheme().pipe(map((theme) => theme?.value == "light")),
);
</script>
