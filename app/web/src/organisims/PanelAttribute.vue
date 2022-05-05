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
      <div class="flex flex-row items-center justify-between flex-grow">
        <div class="flex flex-row">
          <div v-if="componentList" class="min-w-max">
            <SiSelect
              id="nodeSelect"
              v-model="selectedComponentId"
              size="xs"
              name="nodeSelect"
              class="pl-1"
              :value-as-number="true"
              :options="componentList"
              :disabled="!isPinned"
            />
          </div>
          <LockButton v-model="isPinned" class="flex items-center" />
        </div>

        <div class="flex flex-row items-center">
          <SiButtonIcon
            tooltip-text="Attributes"
            :selected="activeView === 'attribute'"
            @click="setActiveView('attribute')"
          >
            <ClipboardListIcon />
          </SiButtonIcon>
          <SiButtonIcon
            tooltip-text="Code"
            :selected="activeView === 'code'"
            @click="setActiveView('code')"
          >
            <CodeIcon />
          </SiButtonIcon>
          <SiButtonIcon
            tooltip-text="Qualifications"
            :selected="activeView === 'qualification'"
            @click="setActiveView('qualification')"
          >
            <CheckCircleIcon />
          </SiButtonIcon>
        </div>

        <div class="flex flex-row items-center">
          <!--
          <SiButtonIcon
            tooltip-text="Connection Viewer (not implemented yet)"
            :color="activeView === 'connection' ? 'cyan' : 'white'"
            @click="setActiveView('connection')"
          >
            <LinkIcon />
          </SiButtonIcon>
          -->
          <!--
          <SiButtonIcon
            tooltip-text="Discovery Viewer (not implemented yet)"
            :color="activeView === 'discovery' ? 'cyan' : 'white'"
            @click="setActiveView('discovery')"
          >
            <AtSymbolIcon />
          </SiButtonIcon>
          -->
          <SiButtonIcon
            tooltip-text="Provider Viewer"
            :selected="activeView === 'provider'"
            @click="setActiveView('provider')"
          >
            <BeakerIcon />
          </SiButtonIcon>
        </div>

        <div class="flex items-center">
          <!--
          <SiButtonIcon
            tooltip-text="Action Viewer (not implemented yet)"
            :color="activeView === 'action' ? 'cyan' : 'white'"
            @click="setActiveView('action')"
          >
            <PlayIcon />
          </SiButtonIcon>
          -->
          <SiButtonIcon
            tooltip-text="Resources"
            :selected="activeView === 'resource'"
            @click="setActiveView('resource')"
          >
            <CubeIcon />
          </SiButtonIcon>
        </div>
      </div>
    </template>

    <template #content>
      <div
        v-if="selectedComponentIdentification"
        class="flex flex-row w-full h-full overflow-auto"
        @click="attributeViewerClick"
      >
        <AttributeViewer
          v-if="activeView === 'attribute'"
          :key="attributeViewerKey"
          :component-id="selectedComponentIdentification.componentId"
          :component-identification="selectedComponentIdentification"
        />
        <QualificationViewer
          v-else-if="activeView === 'qualification'"
          :component-id="selectedComponentIdentification.componentId"
        />
        <ResourceViewer
          v-else-if="activeView === 'resource'"
          :component-id="selectedComponentIdentification.componentId"
        />
        <CodeViewer
          v-else-if="activeView === 'code'"
          :component-id="selectedComponentIdentification.componentId"
        />
        <ProviderViewer
          v-else-if="activeView === 'provider'"
          :schema-variant-id="selectedComponentIdentification.schemaVariantId"
        />
        <div v-else-if="activeView === 'connection'">
          ActiveView "{{ activeView }}" not implemented
        </div>
        <div v-else-if="activeView === 'discovery'">
          ActiveView "{{ activeView }}" not implemented
        </div>
        <div v-else-if="activeView === 'action'">
          ActiveView "{{ activeView }}" not implemented
        </div>
        <div
          v-else
          class="flex flex-col items-center justify-center w-full h-full align-middle"
        >
          <img width="300" :src="cheechSvg" alt="Cheech and Chong!" />
        </div>
      </div>
      <div
        v-else
        class="flex flex-row w-full h-full overflow-auto"
        @click="attributeViewerClick"
      ></div>
    </template>
  </Panel>
</template>

<script setup lang="ts">
import Panel from "@/molecules/Panel.vue";
import LockButton from "@/atoms/LockButton.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import { computed, ref } from "vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { refFrom, untilUnmounted } from "vuse-rx";
import * as Rx from "rxjs";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import AttributeViewer from "@/organisims/AttributeViewer.vue";
import ProviderViewer from "@/organisims/ProviderViewer.vue";
import QualificationViewer from "@/organisims/QualificationViewer.vue";
import ResourceViewer from "@/organisims/ResourceViewer.vue";
import CodeViewer from "@/organisims/CodeViewer.vue";
import _ from "lodash";
import cheechSvg from "@/assets/images/cheech-and-chong.svg";
import { lastSelectedNode$ } from "./SchematicViewer/state";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { schematicData$ } from "./SchematicViewer/Viewer/scene/observable";
import { visibility$ } from "@/observable/visibility";
import { PanelAttributeSubType } from "./PanelTree/panel_types";
import { ChangeSetService } from "@/service/change_set";
import { editButtonPulseUntil$ } from "@/observable/change_set";
import {
  CheckCircleIcon,
  ClipboardListIcon,
  CubeIcon,
  CodeIcon,
  BeakerIcon,
} from "@heroicons/vue/solid";

const randomString = () => `${Math.floor(Math.random() * 50000)}`;
const attributeViewerKey = ref(randomString());
const isPinned = ref<boolean>(false);
const selectedComponentId = ref<number | "">("");

let visibilityChanged = false;
visibility$.pipe(untilUnmounted).subscribe(() => (visibilityChanged = true));

schematicData$.pipe(untilUnmounted).subscribe((schematic) => {
  if (!schematic || selectedComponentId.value === "") {
    visibilityChanged = false;
    return;
  }

  for (const node of schematic.nodes) {
    if (selectedComponentId.value === node.kind.componentId) {
      // Horrible hack to ensure we refetch the edit fields when visibility changes
      // It will flash the screen, but I don't see a better way right now (I'm fixing other schematic panel bugs)
      if (visibilityChanged) {
        attributeViewerKey.value = randomString();
        visibilityChanged = false;
      }
      return;
    }
  }
  visibilityChanged = false;

  isPinned.value = false;
  selectedComponentId.value = "";
});

lastSelectedNode$.pipe(untilUnmounted).subscribe((node) => {
  if (isPinned.value) return;

  const componentId = node?.nodeKind?.componentId;

  // Ignores deselection and fake nodes, as they don't have any attributes
  if (!componentId || componentId === -1) return;

  selectedComponentId.value = componentId;
});
// Re-selects so our observable gets it
Rx.firstValueFrom(lastSelectedNode$).then((last) =>
  lastSelectedNode$.next(last),
);

const props = defineProps<{
  panelIndex: number;
  panelRef: string;
  panelContainerRef: string;
  initialMaximizedFull?: boolean;
  initialMaximizedContainer?: boolean;
  isVisible?: boolean;
  isMaximizedContainerEnabled?: boolean;
  kind: PanelAttributeSubType | null;
}>();

const activeView = ref<string>(props.kind ?? "attribute");
const setActiveView = (view: string) => {
  activeView.value = view;
};

const componentIdentificationList = refFrom<
  LabelList<ComponentIdentification | "">
>(
  ComponentService.listComponentsIdentification().pipe(
    Rx.switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([[]]);
      } else {
        const list: LabelList<ComponentIdentification | ""> = _.cloneDeep(
          response.list,
        );
        list.push({ label: "", value: "" });
        return Rx.from([list]);
      }
    }),
  ),
);

const componentList = computed((): LabelList<number | ""> => {
  let list: LabelList<number | ""> = [];
  if (componentIdentificationList.value) {
    for (const item of componentIdentificationList.value) {
      let value: number | "" = "";
      if (item.value !== "") {
        value = item.value.componentId;
      }
      list.push({ label: item.label, value: value });
    }
  }
  return list;
});

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

const editMode = refFrom(ChangeSetService.currentEditMode());
const attributeViewerClick = () => {
  if (activeView.value === "attribute" && !editMode.value) {
    editButtonPulseUntil$.next(new Date(new Date().getTime() + 5000));
  }
};
</script>

<style scoped>
.menu-button-active {
  color: #69e3d2;
}

.menu-button-inactive {
  color: #c6c6c6;
}

.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
