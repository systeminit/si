<template>
  <Panel
    :panel-index="panelIndex"
    :panel-ref="panelRef"
    :panel-container-ref="panelContainerRef"
    :initial-maximized-container="initialMaximizedContainer"
    :initial-maximized-full="initialMaximizedFull"
    :is-visible="isVisible"
    :is-maximized-container-enabled="isMaximizedContainerEnabled"
  >
    <template #content>
      <SchematicViewer />
      <!-- <div
        class="flex flex-col items-center justify-center w-full h-full align-middle"
      >
        {{ panelContainerRef }}
        {{ panelRef }}
        Schematic Panel
        <button @click="getSchematic">Get call</button>
        <button @click="setSchematic">Set call</button>
      </div> -->
    </template>
  </Panel>
</template>

<script setup lang="ts">
import Panel from "@/molecules/Panel.vue";
import SchematicViewer from "@/organisims/SchematicViewer.vue";

// import { SchematicService } from "@/service/schematic";
// import { GlobalErrorService } from "@/service/global_error";
// import { ApiResponse } from "@/api/sdf";
// import { GetSchematicResponse } from "@/service/schematic/get_schematic";
// import { SetSchematicResponse } from "@/service/schematic/set_schematic";


// TODO: Alex, here is your panel. The switcher is fucked, but otherwise, should be good to port.

defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: Boolean,
  isMaximizedContainerEnabled: Boolean,
});

// const getSchematic = () => {
//   SchematicService.getSchematic({ context: "poop" }).subscribe(
//     (response: ApiResponse<GetSchematicResponse>) => {
//       if (response.error) {
//         GlobalErrorService.set(response);
//       }
//       console.log("get response", { response });
//     },
//   );
// };

// const setSchematic = () => {
//   SchematicService.setSchematic({ name: "canoe" }).subscribe(
//     (response: ApiResponse<SetSchematicResponse>) => {
//       if (response.error) {
//         GlobalErrorService.set(response);
//       }
//       console.log("set response", { response });
//     },
//   );
// };
</script>

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
