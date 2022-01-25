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

      <div class="min-w-max">
        <button @click="setToAttribute">
          <VueFeather type="home" stroke="grey" size="1.5rem" />
        </button>
      </div>

      <div class="min-w-max">
        <button @click="setToQualification">
          <VueFeather type="crosshair" stroke="grey" size="1.5rem" />
        </button>
      </div>

      <div class="min-w-max">
        <button @click="setToResource">
          <VueFeather type="box" stroke="grey" size="1.5rem" />
        </button>
      </div>

      <LockButton v-model="isPinned" />
    </template>

    <template #content>
      <AttributeViewer
        v-if="selectedComponentId && activeView === 'attribute'"
        :component-id="selectedComponentId"
      />
      <QualificationViewer
        v-if="selectedComponentId && activeView === 'qualification'"
        :component-id="selectedComponentId"
      />
      <ResourceViewer
        v-if="selectedComponentId && activeView === 'resource'"
        :component-id="selectedComponentId"
      />
      <div
        v-if="!selectedComponentId"
        class="flex flex-col items-center justify-center w-full h-full align-middle"
      >
        <img width="300" :src="cheechSvg" />
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

const props = defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: Boolean,
  isMaximizedContainerEnabled: Boolean,
});

const activeView = ref<string>("attribute");
const setToResource = () => {
  activeView.value = "resource";
};
const setToAttribute = () => {
  activeView.value = "attribute";
};
const setToQualification = () => {
  activeView.value = "qualification";
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
