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

      <div
        v-if="schematicKind === SchematicKind.Component"
        class="flex flex-row"
      >
        <div v-if="componentNamesOnlyList" class="min-w-max">
          <SiSelect
            id="nodeSelect"
            v-model="selectedComponentId"
            size="xs"
            name="nodeSelect"
            class="pl-1"
            :value-as-number="true"
            :options="componentNamesOnlyList"
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
        :deployment-component-pin="
          selectedComponentId ? selectedComponentId : undefined
        "
        :is-component-panel-pinned="isPinned"
      />
    </template>
  </Panel>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import { ComponentService } from "@/service/component";
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
import { refFrom, untilUnmounted } from "vuse-rx";
import { switchMap } from "rxjs/operators";
import { ChangeSetService } from "@/service/change_set";
import { NodeAddEvent, ViewerEventObservable } from "./SchematicViewer/event";
import { deploymentSelection$, SelectedNode } from "./SchematicViewer/state";
import { visibility$ } from "@/observable/visibility";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { firstValueFrom } from "rxjs";
import _ from "lodash";
import * as Rx from "rxjs";
import * as MODEL from "./SchematicViewer/model";

const selectedComponentId = ref<number | "">("");
const isPinned = ref<boolean>(false);

// We garantee that the latest update will always be the last element in the list
deploymentSelection$.pipe(untilUnmounted).subscribe((selections) => {
  if (isPinned.value) return;

  const last = selections?.length
    ? selections[selections.length - 1]
    : undefined;
  const componentId = last?.nodes?.length
    ? last.nodes[0]?.nodeKind?.componentId
    : undefined;

  // Ignores fake nodes as they don't have any attributes
  if (componentId === -1) return;

  selectedComponentId.value = componentId ?? "";
});

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

const applicationId = refFrom<number | null>(
  ApplicationService.currentApplication().pipe(
    switchMap((application) => {
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
const selectedDeployment = refFrom<SelectedNode[]>(
  deploymentSelection$.asObservable(),
);

let rootDeployment = ref<SelectedNode | null>(null);
deploymentSelection$.pipe(untilUnmounted).subscribe((selections) => {
  if (!isPinned.value) {
    const selection = selections.find(
      (sel) => sel.parentDeploymentNodeId === null,
    );
    if (!selection) {
      rootDeployment.value = null;
      return;
    }

    // We have to clone otherwise changes to the underlying selected node will alter us in a way we don't expect
    rootDeployment.value = {
      parentDeploymentNodeId: selection.parentDeploymentNodeId,
      nodes: [...selection.nodes],
    };
  }
});

visibility$.pipe(untilUnmounted).subscribe((_) => {
  isPinned.value = false;
  rootDeployment.value = null;
  selectedComponentId.value = "";
});

const addMenuEnabled = computed(() => {
  switch (schematicKind.value) {
    case SchematicKind.Component:
      return editMode.value && !!rootDeployment.value?.nodes?.length;
    case SchematicKind.Deployment:
      return editMode.value;
  }
  throw new Error(`unsupported schematic kind ${schematicKind.value}`);
});

const addNode = async (schemaId: number, _event: MouseEvent) => {
  const response = await firstValueFrom(
    SchematicService.getNodeTemplate({ schemaId }),
  );
  if (response.error) {
    GlobalErrorService.set(response);
    return;
  }

  let deployment;
  switch (schematicKind.value) {
    case SchematicKind.Component:
      deployment = rootDeployment.value;
      break;
    case SchematicKind.Deployment:
      deployment = (selectedDeployment.value ?? [])[0] ?? null;
      break;
  }

  const n = MODEL.fakeNodeFromTemplate(
    response,
    (deployment?.nodes ?? [])[0]?.id,
  );
  const event = new NodeAddEvent({ node: n, schemaId: schemaId });

  viewerEventObservable.viewerEvent$.next(event);
};

const componentNamesOnlyList = refFrom<LabelList<number | "">>(
  ComponentService.listComponentsNamesOnly().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([[]]);
      } else {
        const list: LabelList<number | ""> = _.cloneDeep(response.list);
        list.push({ label: "", value: "" });
        return Rx.from([list]);
      }
    }),
  ),
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
