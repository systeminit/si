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
          <div v-if="componentNamesOnlyList" class="min-w-max">
            <SiSelect
              id="nodeSelect"
              v-model="selectedComponentId"
              size="xs"
              name="nodeSelect"
              class="pl-1"
              :value-as-number="true"
              :options="componentNamesOnlyList"
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

    <template #content>
      <!-- NOTE(nick): CLion's Vue.js plugin version 213.6461.23 shows an incorrect warning message here.
      Essentially, the IDE will say that "selectComponentId" can still be an empty string despite the "v-if" directive
      checking the "truthiness" of it (either in the div or the viewer declarations). Due to the usage of the directive,
      we know it will be a number and the warning is incorrect.
      For more information: https://v3.vuejs.org/guide/conditional.html#v-if
      -->
      <div
        v-if="selectedComponentId"
        class="flex flex-row w-full h-full overflow-auto"
      >
        <AttributeViewer
          v-if="activeView === 'attribute'"
          :component-id="selectedComponentId"
        />
        <QualificationViewer
          v-else-if="activeView === 'qualification'"
          :component-id="selectedComponentId"
        />
        <ResourceViewer
          v-else-if="activeView === 'resource'"
          :component-id="selectedComponentId"
        />
        <div v-else-if="activeView === 'code'">
          ActiveView "{{ activeView }}" not implemented
        </div>
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
import { ref } from "vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { refFrom } from "vuse-rx";
import { switchMap } from "rxjs/operators";
import { from } from "rxjs";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import AttributeViewer from "@/organisims/AttributeViewer.vue";
import QualificationViewer from "@/organisims/QualificationViewer.vue";
import ResourceViewer from "@/organisims/ResourceViewer.vue";
import VueFeather from "vue-feather";
import _ from "lodash";
import cheechSvg from "@/assets/images/cheech-and-chong.svg";

const isPinned = ref<boolean>(false);
const selectedComponentId = ref<number | "">("");

const props = defineProps<{
  panelIndex: number;
  panelRef: string;
  panelContainerRef: string;
  initialMaximizedFull?: boolean;
  initialMaximizedContainer?: boolean;
  isVisible?: boolean;
  isMaximizedContainerEnabled?: boolean;
}>();

const activeView = ref<string>("attribute");
const setActiveView = (view: string) => {
  activeView.value = view;
};

const componentNamesOnlyList = refFrom<LabelList<number | "">>(
  ComponentService.listComponentsNamesOnly().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        const list: LabelList<number | ""> = _.cloneDeep(response.list);
        list.push({ label: "", value: "" });
        return from([list]);
      }
    }),
  ),
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
