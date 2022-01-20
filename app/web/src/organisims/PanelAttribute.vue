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
      <div v-if="componentNamesOnlyList" class="min-w-max">
        <SiSelect
          id="nodeSelect"
          v-model="selectedComponentId"
          size="xs"
          name="nodeSelect"
          class="pl-1"
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

      <LockButton v-model="isPinned" />
    </template>

    <template #content>
      <!-- FIXME(nick): there is a bug unrelated to the viewer buttons where EditFields will be undefined despite the component ID being valid -->
      <AttributeViewer
        v-if="selectedComponentId && activeView === 'attribute'"
        :component-id="selectedComponentId"
      />
      <QualificationViewer
        v-if="selectedComponentId && activeView === 'qualification'"
        :component-id="selectedComponentId"
      />
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
import VueFeather from "vue-feather";

const isPinned = ref<boolean>(false);
const selectedComponentId = ref<number | undefined>(undefined);

defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: Boolean,
  isMaximizedContainerEnabled: Boolean,
});

const activeView = ref<string>("attribute");
const setToAttribute = () => {
  activeView.value = "attribute";
};
const setToQualification = () => {
  activeView.value = "qualification";
};

const componentNamesOnlyList = refFrom<LabelList<number>>(
  ComponentService.listComponentsNamesOnly().pipe(
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response.list]);
      }
    }),
  ),
);
</script>
