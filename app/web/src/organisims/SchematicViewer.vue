<template>
  <div :id="viewerId" class="w-full h-full">
    <Viewer
      ref="viewer"
      :schematic-viewer-id="viewerId"
      :viewer-state="viewerState"
      :schematic-data="schematicData"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, defineExpose } from "vue";
import _ from "lodash";

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

const componentName = "SchematicViewer";
const componentId = _.uniqueId();

const viewer = ref<typeof Viewer | null>(null);
const viewerId = componentName + "-" + componentId;
const viewerState = new ViewerStateMachine();

// Ref<Schematic>
let schematicData = ref<Schematic>();

onMounted(() => {
  getSchematic();
});

const getSchematic = () => {
  SchematicService.getSchematic().subscribe(
    (response: ApiResponse<GetSchematicResponse>) => {
      if (response.error) {
        GlobalErrorService.set(response);
      } else {
        schematicData.value = response;
      }
    },
  );
};

function addNode(schemaId: number) {
  if (viewer.value) {
    viewer.value.handleNodeAdd(schemaId);
  }
}

defineExpose({
  addNode,
});
</script>
