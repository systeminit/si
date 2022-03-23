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
    />
  </div>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import _ from "lodash";
import * as Rx from "rxjs";

import Viewer from "./SchematicViewer/Viewer.vue";

import { ViewerStateMachine } from "./SchematicViewer/state";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
// import { SetSchematicResponse } from "@/service/schematic/set_schematic";

// import { schematicData } from "./SchematicViewer/model";
// import { schematicData$ } from "./SchematicViewer/data";
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
// export interface ViewerData {
//   component: {
//     id: string;
//   };
//   viewer: {
//     id: string;
//     element: HTMLElement | null;
//   };
//   state: ViewerStateMachine;
// }

// const resizeEvent = ref<null | ResizeEvent>(null);
// const ticking = ref<boolean>(false);
// const maximizedData = ref<PanelMaximized | null>(null);
// const panelSelector = ref<Array<typeof PanelSelector>>([]);
// const panelSize = ref<
//   Record<string, { width: number; height: number; hidden: boolean }>
// >({});

const props = defineProps({
  viewerEvent$: {
    type: Object as PropType<Rx.ReplaySubject<ViewerEvent | null>> | undefined,
    required: false,
    default: undefined,
  },
  schematicKind: {
    type: String as PropType<SchematicKind | null>,
    required: true,
  },
});

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
