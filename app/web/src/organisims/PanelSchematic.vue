<template>
  <Panel
    :panel-index="props.panelIndex"
    :panel-ref="props.panelRef"
    :panel-container-ref="props.panelContainerRef"
    :initial-maximized-container="props.initialMaximizedContainer"
    :initial-maximized-full="props.initialMaximizedFull"
    :is-visible="props.isVisible"
    :is-maximized-container-enabled="props.isMaximizedContainerEnabled"
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

      <div
        v-if="schematicKind === SchematicKind.Component"
        class="flex flex-row"
      >
        <div v-if="deploymentComponentsList" class="min-w-max">
          <SiSelect
            id="nodeSelect"
            v-model="selectedDeploymentComponentId"
            size="xs"
            name="nodeSelect"
            class="pl-1"
            :value-as-number="true"
            :options="deploymentComponentsList"
            :disabled="!isPinned"
          />
        </div>
        <LockButton v-model="isPinned" />
      </div>

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
        :deployment-node-pin="selectedDeploymentNodeId ?? undefined"
        :is-component-panel-pinned="isPinned"
        :schematic-data="schematicData ?? null"
      />
    </template>
  </Panel>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";

import { ComponentService } from "@/service/component";
import Panel from "@/molecules/Panel.vue";
import SchematicViewer from "@/organisims/SchematicViewer.vue";
import SiSelect from "@/atoms/SiSelect.vue";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";

import { schematicData$ } from "./SchematicViewer/Viewer/scene/observable";
import { Schematic } from "./SchematicViewer/model";
import {
  SchematicKind,
  MenuFilter,
  schematicKindFromString,
} from "@/api/sdf/dal/schematic";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { LabelList } from "@/api/sdf/dal/label_list";
import LockButton from "@/atoms/LockButton.vue";
import NodeAddMenu from "@/molecules/NodeAddMenu.vue";
import { ApplicationService } from "@/service/application";
import { refFrom, untilUnmounted } from "vuse-rx";
import { ChangeSetService } from "@/service/change_set";
import { NodeAddEvent, ViewerEventObservable } from "./SchematicViewer/event";
import { lastSelectedDeploymentNode$ } from "./SchematicViewer/state";
import _ from "lodash";
import * as Rx from "rxjs";
import * as MODEL from "./SchematicViewer/model";

const schematicData = refFrom<Schematic | null>(schematicData$);

const isPinned = ref<boolean>(false);
const selectedDeploymentNodeId = ref<number | null>(null);
const selectedDeploymentComponentId = ref<number | "">("");

watch(selectedDeploymentComponentId, (componentId) => {
  if (!componentId || !schematicData.value) {
    selectedDeploymentNodeId.value = null;
    return;
  }

  for (const node of schematicData.value.nodes) {
    if (node.kind.componentId === componentId) {
      selectedDeploymentNodeId.value = node.id;
      return;
    }
  }
  throw new Error(`Node wasn't found ${componentId}`);
});

lastSelectedDeploymentNode$.pipe(untilUnmounted).subscribe((node) => {
  if (!schematicData.value) return;

  if (isPinned.value) return;

  const componentId = node?.nodeKind?.componentId;

  // Ignores fake nodes as they don't have any attributes
  if (componentId === -1) return;

  selectedDeploymentComponentId.value = componentId ?? "";
});

// Re-selects so our observable gets it
Rx.firstValueFrom(lastSelectedDeploymentNode$).then((last) =>
  lastSelectedDeploymentNode$.next(last),
);

schematicData$.pipe(untilUnmounted).subscribe((schematic) => {
  if (!schematic || selectedDeploymentComponentId.value === "") return;

  for (const node of schematic.nodes) {
    if (selectedDeploymentComponentId.value === node.kind.componentId) {
      return;
    }
  }

  isPinned.value = false;
  selectedDeploymentComponentId.value = "";
  selectedDeploymentNodeId.value = null;
});

const viewerEventObservable = new ViewerEventObservable();

const props = defineProps<{
  panelIndex: number;
  panelRef: string;
  panelContainerRef: string;
  initialMaximizedFull: boolean;
  initialMaximizedContainer: boolean;
  isVisible: boolean;
  isMaximizedContainerEnabled: boolean;
  kind: SchematicKind | null;
}>();

const schematicKindRaw = ref<string>(props.kind ?? SchematicKind.Deployment);

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

const applicationId = refFrom<number | null>(
  ApplicationService.currentApplication().pipe(
    Rx.switchMap((application) => {
      if (application) {
        return Rx.from([application.id]);
      } else {
        return Rx.from([null]);
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

const addMenuEnabled = computed(() => {
  switch (schematicKind.value) {
    case SchematicKind.Component:
      return editMode.value && selectedDeploymentComponentId.value;
    case SchematicKind.Deployment:
      return editMode.value;
  }
  throw new Error(`unsupported schematic kind ${schematicKind.value}`);
});

const addNode = async (schemaId: number, _event: MouseEvent) => {
  const response = await Rx.firstValueFrom(
    SchematicService.getNodeTemplate({ schemaId }),
  );
  if (response.error) {
    GlobalErrorService.set(response);
    return;
  }

  const n = MODEL.fakeNodeFromTemplate(
    response,
    selectedDeploymentNodeId.value,
  );
  const event = new NodeAddEvent({ node: n, schemaId: schemaId });

  viewerEventObservable.viewerEvent$.next(event);
};

const componentIdentificationList = refFrom<LabelList<ComponentIdentification>>(
  ComponentService.listComponentsIdentification().pipe(
    Rx.switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([[]]);
      } else {
        return Rx.from([response.list]);
      }
    }),
  ),
);

const deploymentComponentsList = computed(
  (): LabelList<number | ""> => {
    let list: LabelList<number | ""> = [];
    if (componentIdentificationList.value) {
      for (const item of componentIdentificationList.value) {
        if (item.value.schematicKind === SchematicKind.Deployment) {
          list.push({ label: item.label, value: item.value.componentId });
        }
      }
      list.push({ label: "", value: "" });
    }
    return list;
  },
);
</script>

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
