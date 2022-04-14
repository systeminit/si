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
          <LockButton v-model="isPinned" />
        </div>

        <!-- NOTE(nick): old-web adds items-center for this div.
        More information: https://tailwindcss.com/docs/align-items#center
        -->
        <div class="flex flex-row items-center">
          <button
            class="pl-1 focus:outline-none"
            @click="setActiveView('attribute')"
          >
            <VueFeather
              v-if="activeView === 'attribute'"
              type="disc"
              stroke="cyan"
              size="1.1em"
            />
            <VueFeather v-else type="disc" class="text-gray-300" size="1.1em" />
          </button>
          <button
            class="pl-1 focus:outline-none"
            @click="setActiveView('code')"
          >
            <VueFeather
              v-if="activeView === 'code'"
              type="code"
              stroke="cyan"
              size="1.1em"
            />
            <VueFeather v-else type="code" class="text-gray-300" size="1.1em" />
          </button>
          <button
            class="pl-1 focus:outline-none"
            @click="setActiveView('qualification')"
          >
            <VueFeather
              v-if="activeView === 'qualification'"
              type="check-square"
              stroke="cyan"
              size="1.1em"
            />
            <VueFeather
              v-else
              type="check-square"
              class="text-gray-300"
              size="1.1em"
            />
          </button>
        </div>

        <!-- NOTE(nick): old-web adds text-white for buttons in this div.
        More information: https://tailwindcss.com/docs/text-color
        -->
        <div class="flex flex-row items-center">
          <button
            class="pl-1 text-white focus:outline-none"
            @click="setActiveView('connection')"
          >
            <VueFeather
              v-if="activeView === 'connection'"
              type="link-2"
              stroke="cyan"
              size="1.1em"
            />
            <VueFeather
              v-else
              type="link-2"
              class="text-gray-300"
              size="1.1em"
            />
          </button>
          <button
            class="pl-1 text-white focus:outline-none"
            @click="setActiveView('discovery')"
          >
            <VueFeather
              v-if="activeView === 'discovery'"
              type="at-sign"
              stroke="cyan"
              size="1.1em"
            />
            <VueFeather
              v-else
              type="at-sign"
              class="text-gray-300"
              size="1.1em"
            />
          </button>
        </div>

        <!-- NOTE(nick): old-web removes flex-row in this div and adds text-white for buttons in this div.
        More information: https://tailwindcss.com/docs/flex-direction#row
        -->
        <div class="flex items-center">
          <button
            class="pl-1 text-white focus:outline-none"
            @click="setActiveView('action')"
          >
            <VueFeather
              v-if="activeView === 'action'"
              type="play"
              stroke="cyan"
              size="1.1em"
            />
            <VueFeather v-else type="play" class="text-gray-300" size="1.1em" />
          </button>
          <button
            class="pl-1 text-white focus:outline-none"
            @click="setActiveView('resource')"
          >
            <VueFeather
              v-if="activeView === 'resource'"
              type="box"
              stroke="cyan"
              size="1.1em"
            />
            <VueFeather v-else type="box" class="text-gray-300" size="1.1em" />
          </button>
        </div>
      </div>
    </template>

    <template v-if="selectedComponentIdentification" #content>
      <!-- NOTE(nick): CLion's Vue.js plugin version 213.6461.23 shows an incorrect warning message here.
      Essentially, the IDE will say that "selectComponentIdentification.componentId" can still be null despite the "v-if" directive
      checking the "truthiness" of it (either in the div or the viewer declarations). Due to the usage of the directive,
      we know it will be a number and the warning is incorrect.
      For more information: https://v3.vuejs.org/guide/conditional.html#v-if
      -->

      <!-- NOTE(paulo): `npm run type-check` complains about selectedComponetnIdentification.componentId being null
      if we only check in the template, but checking in the template ensures we can't endup rendering two #content slots
      so we check on both places
      -->
      <div
        v-if="selectedComponentIdentification"
        class="flex flex-row w-full h-full overflow-auto"
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
    </template>
  </Panel>
</template>

<script setup lang="ts">
import Panel from "@/molecules/Panel.vue";
import LockButton from "@/atoms/LockButton.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import { computed, ref } from "vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { refFrom, untilUnmounted } from "vuse-rx";
import * as Rx from "rxjs";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import AttributeViewer from "@/organisims/AttributeViewer.vue";
import QualificationViewer from "@/organisims/QualificationViewer.vue";
import ResourceViewer from "@/organisims/ResourceViewer.vue";
import CodeViewer from "@/organisims/CodeViewer.vue";
import VueFeather from "vue-feather";
import _ from "lodash";
import cheechSvg from "@/assets/images/cheech-and-chong.svg";
import { lastSelectedNode$ } from "./SchematicViewer/state";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { schematicData$ } from "./SchematicViewer/Viewer/scene/observable";
import { visibility$ } from "@/observable/visibility";
import { PanelAttributeSubType } from "./PanelTree/panel_types";

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

const componentList = computed(
  (): LabelList<number | ""> => {
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
  },
);

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
