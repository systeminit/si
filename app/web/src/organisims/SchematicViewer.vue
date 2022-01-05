<template>
  <div :id="viewerId" ref="viewer" class="w-full h-full">
    <Viewer
      :schematic-viewer-id="viewerId"
      :viewer-state="viewerState"
      :schematic-data="schematicData"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref} from "vue";
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

const viewer = ref(null);
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

//   mounted(): void {
//     this.viewer.element = this.$refs.viewport as HTMLElement;

//     console.log("sending schematic data:", schematicData);
//     schematicData$.next(schematicData);
//   },
// });
</script>
