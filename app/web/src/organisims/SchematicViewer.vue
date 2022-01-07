<template>
  <div :id="viewerId" class="w-full h-full">
    <Viewer
      :schematic-viewer-id="viewerId"
      :viewer-state="viewerState"
      :schematic-data="schematicData"
      :viewer-event$="props.viewerEvent$"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, defineExpose, PropType } from "vue";
import _ from "lodash";
import * as Rx from "rxjs";

import Viewer from "./SchematicViewer/Viewer.vue";

import { ViewerStateMachine } from "./SchematicViewer/state";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { ApiResponse } from "@/api/sdf";
import { GetSchematicResponse } from "@/service/schematic/get_schematic";
// import { SetSchematicResponse } from "@/service/schematic/set_schematic";

// import { schematicData } from "./SchematicViewer/model";
// import { schematicData$ } from "./SchematicViewer/data";
import { Schematic } from "./SchematicViewer/model";
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
});

const componentName = "SchematicViewer";
const componentId = _.uniqueId();

const viewerId = componentName + "-" + componentId;
const viewerState = new ViewerStateMachine();

// Ref<Schematic>
let schematicData = ref<Schematic>();

onMounted(() => {
  getSchematic();
});

const getSchematic = () => {
  SchematicService.getSchematic({ context: "poop" }).subscribe(
    (response: ApiResponse<GetSchematicResponse>) => {
      if (response.error) {
        GlobalErrorService.set(response);
      } else {
        schematicData.value = response;
      }
    },
  );
};
</script>
