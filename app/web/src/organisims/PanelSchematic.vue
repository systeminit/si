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
          tooltip-text="Diagram sub-panel selector"
          name="schematicSelect"
          class="pl-1 w-32"
          :options="schematicKinds"
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
            v-tooltip.bottom="{
              content:
                'Node selection scape-hatch.<br/>For when you need to change nodes while the panel is locked.<br/>If unlocked, just use the Deployment Diagram sub-panel for selection.',
              html: true,
            }"
            name="nodeSelect"
            class="pl-1 w-32"
            :value-as-number="true"
            :options="deploymentComponentsList"
            :disabled="!isPinned"
          />
        </div>
        <LockButton v-model="isPinned" class="flex items-center pl-1" />
      </div>

      <NodeAddMenu
        v-if="addMenuFilters"
        class="ml-4"
        :add-to="
          schematicKind === SchematicKind.Deployment
            ? `application`
            : selectedDeploymentNode?.title
        "
        :filter="addMenuFilters"
        :disabled="!addMenuEnabled"
        @selected="addNode"
      />
    </template>
    <template #content>
      <SchematicViewer
        :viewer-event$="viewerEventObservable.viewerEvent$"
        :schematic-kind="schematicKind"
        :deployment-node-selected="
          schematicKind === SchematicKind.Component
            ? selectedDeploymentNode?.id ?? null
            : null
        "
        :schematic-data="schematicData ?? null"
        :adding-node="addingNode"
      />
    </template>
  </Panel>
</template>

<script setup lang="ts">
import _ from "lodash";
import { Node } from "@/organisims/SchematicViewer/Viewer/obj/node";
import * as Rx from "rxjs";

import { computed, ref, watch } from "vue";

import { ComponentService } from "@/service/component";
import Panel from "@/molecules/Panel.vue";
import SchematicViewer from "@/organisims/SchematicViewer.vue";
import SiSelect from "@/atoms/SiSelect.vue";

import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";

import { schematicData$ } from "@/observable/schematic";
import { Schematic, SchematicNode } from "@/api/sdf/dal/schematic";
import { visibility$ } from "@/observable/visibility";
import {
  SchematicKind,
  schematicKindFromString,
} from "@/api/sdf/dal/schematic";
import { MenuFilter } from "@/api/sdf/dal/menu";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { LabelList } from "@/api/sdf/dal/label_list";
import LockButton from "@/atoms/LockButton.vue";
import NodeAddMenu from "@/molecules/NodeAddMenu.vue";
import { ApplicationService } from "@/service/application";
import { refFrom, untilUnmounted } from "vuse-rx";
import { ChangeSetService } from "@/service/change_set";
import {
  NodeAddEvent,
  ViewerEventObservable,
} from "./SchematicViewer/viewer_event";
import { lastSelectedDeploymentNode$ } from "@/observable/selection";

const schematicData = refFrom<Schematic | null>(schematicData$);

const isPinned = ref<boolean>(false);
const selectedDeploymentNode = ref<SchematicNode | null>(null);
const selectedDeploymentComponentId = ref<number | "">("");

watch(selectedDeploymentComponentId, (componentId) => {
  if (!componentId || !schematicData.value) {
    selectedDeploymentNode.value = null;
    return;
  }

  for (const node of schematicData.value.nodes) {
    if (node.kind.componentId === componentId) {
      selectedDeploymentNode.value = node;
      return;
    }
  }
  throw new Error(`Node wasn't found ${componentId}`);
});

const updateDeploymentSelection = (node: Node | null) => {
  const componentId = node?.nodeKind?.componentId;

  if (!schematicData.value) return;
  // Locked panels can't change selection by clicking in nodes
  if (isPinned.value) return;
  // Ignores deselection and fake nodes, as they don't have any attributes
  if (!componentId || componentId === -1) return;

  selectedDeploymentComponentId.value = componentId ?? "";
};

lastSelectedDeploymentNode$
  .pipe(untilUnmounted)
  .subscribe((node) => updateDeploymentSelection(node));

schematicData$.pipe(untilUnmounted).subscribe((schematic) => {
  if (!schematic || selectedDeploymentComponentId.value === "") return;

  for (const node of schematic.nodes) {
    if (selectedDeploymentComponentId.value === node.kind.componentId) {
      return;
    }
  }

  isPinned.value = false;
  selectedDeploymentComponentId.value = "";
  selectedDeploymentNode.value = null;
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

const schematicKind = computed<SchematicKind>(() =>
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

const addingNode = ref(false);
visibility$.pipe(untilUnmounted).subscribe(() => {
  addingNode.value = false;
});

const addNode = async (schemaId: number, _event: MouseEvent) => {
  addingNode.value = true;
  const template = await Rx.firstValueFrom(
    SchematicService.getNodeTemplate({ schemaId }),
  );
  if (template.error) {
    GlobalErrorService.set(template);
    return;
  }

  // Generates fake node from template
  const node = {
    id: -1,
    kind: { kind: template.kind, componentId: -1 },
    title: template.title,
    name: template.name,
    positions: [
      {
        schematicKind:
          template.kind === "component"
            ? SchematicKind.Component
            : SchematicKind.Deployment,
        deploymentNodeId: selectedDeploymentNode.value?.id,
        x: 350,
        y: 0,
      },
    ],
    schemaVariantId: template.schemaVariantId,
  };
  const event = new NodeAddEvent({ node, schemaId: schemaId });

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

const deploymentComponentsList = computed((): LabelList<number | ""> => {
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
});
</script>

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
