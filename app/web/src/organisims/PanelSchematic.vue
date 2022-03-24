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
    <template #menuButtons>
      <div class="min-w-max">
        <SiSelect
          id="schematicSelect"
          v-model="schematicKindRaw"
          size="xs"
          name="schematicSelect"
          class="pl-1"
          :options="schematicKinds"
          :styling="schematicSelectorStyling"
        />
      </div>

      <div class="min-w-max">
        <SiSelect
          v-if="schematicKind === SchematicKind.Deployment"
          id="systemSelect"
          size="xs"
          name="systemSelect"
          class="pl-1"
          :options="systemsList"
          :styling="schematicSelectorStyling"
        />
      </div>

      <LockButton
        v-if="schematicKind === SchematicKind.Component"
        v-model="isPinned"
      />

      <NodeAddMenu
        v-if="addMenuFilters"
        class="pl-4"
        :filter="addMenuFilters"
        :disabled="!addMenuEnabled"
        @selected="addNode"
      />
    </template>
    <template #content>
      <SchematicViewer
        :viewer-event$="viewerEventObservable.viewerEvent$"
        :schematic-kind="schematicKind"
      />
      <!-- <SchematicViewer /> -->
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
import { computed, ref } from "vue";

import Panel from "@/molecules/Panel.vue";
import SchematicViewer from "@/organisims/SchematicViewer.vue";
import SiSelect from "@/atoms/SiSelect.vue";

import {
  SchematicKind,
  MenuFilter,
  schematicKindFromString,
} from "@/api/sdf/dal/schematic";
import { LabelList } from "@/api/sdf/dal/label_list";
import LockButton from "@/atoms/LockButton.vue";
import NodeAddMenu from "@/molecules/NodeAddMenu.vue";
import { ApplicationService } from "@/service/application";
import { refFrom } from "vuse-rx";
import { switchMap } from "rxjs/operators";
import { from } from "rxjs";
import { ChangeSetService } from "@/service/change_set";
import { NodeAddEvent, ViewerEventObservable } from "./SchematicViewer/event";
import { deploymentSelection$ } from "./SchematicViewer/state";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { firstValueFrom } from "rxjs";
import * as MODEL from "./SchematicViewer/model";
import * as OBJ from "./SchematicViewer/Viewer/obj";

// import { SchematicService } from "@/service/schematic";
// import { GlobalErrorService } from "@/service/global_error";
// import { ApiResponse } from "@/api/sdf";
// import { GetSchematicResponse } from "@/service/schematic/get_schematic";
// import { SetSchematicResponse } from "@/service/schematic/set_schematic";

// TODO: Alex, here is your panel. The switcher is fucked, but otherwise, should be good to port.

// TODO: degfine viewer observable here.

const viewerEventObservable = new ViewerEventObservable();

defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: Boolean,
  isMaximizedContainerEnabled: Boolean,
});

const schematicKindRaw = ref<string>(SchematicKind.Deployment);
// const rootObjectId = ref<number | null>(null);

const schematicKind = computed(() =>
  schematicKindFromString(schematicKindRaw.value),
);

const schematicKinds = computed(() => {
  let labels: LabelList<string> = [];
  for (const value of Object.values(SchematicKind)) {
    labels.push({
      label: value,
      value: value,
    });
  }
  return labels;
});

const schematicSelectorStyling = computed(() => {
  let classes: Record<string, boolean> = {};
  classes["bg-selectordark"] = true;
  classes["text-gray-400"] = true;
  classes["border-gray-800"] = true;
  return classes;
});

// TODO: Re-implement systems, and fetch the default system. (adam)
const systemsList = computed(() => {
  return [{ value: "prod", label: "prod" }];
});

const isPinned = ref<boolean>(false);
const applicationId = refFrom<number | null>(
  ApplicationService.currentApplication().pipe(
    switchMap((application) => {
      if (application) {
        return from([application.id]);
      } else {
        return from([null]);
      }
    }),
  ),
);

const addMenuFilters = computed(() => {
  if (applicationId.value) {
    const filter: MenuFilter = {
      rootComponentId: applicationId.value,
      schematicKind: schematicKind.value,
    };
    return filter;
  }
  return null;
});

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());
const rootDeployment = refFrom<Array<OBJ.Node> | null>(
  deploymentSelection$.asObservable(),
);
const addMenuEnabled = computed(() => {
  switch (schematicKind.value) {
    case SchematicKind.Component:
      return editMode.value && rootDeployment.value;
    case SchematicKind.Deployment:
      return editMode.value;
  }
  return editMode.value;
});

const addNode = async (schemaId: number, _event: MouseEvent) => {
  const response = await firstValueFrom(
    SchematicService.getNodeTemplate({ schemaId }),
  );
  if (response.error) {
    GlobalErrorService.set(response);
    return;
  }

  const n = MODEL.fakeNodeFromTemplate(response);
  const event = new NodeAddEvent({ node: n, schemaId: schemaId });

  viewerEventObservable.viewerEvent$.next(event);
};

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

// onMounted(() => {
//   // console.log("aaaaaaaaa:", schematicViewer.value);
// });
</script>

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
