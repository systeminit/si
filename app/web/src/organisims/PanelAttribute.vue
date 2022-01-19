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

      <LockButton v-model="isPinned" />
    </template>

    <template #content>
      <AttributeViewer
        v-if="selectedComponentId"
        :component-id="selectedComponentId"
      />
      <QualificationViewer
        v-if="selectedComponentId"
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

const isPinned = ref<boolean>(false);
const selectedComponentId = ref<number | undefined>(undefined);
// const attributeViewer = ref<typeof AttributeViewer | null>(null);

defineProps({
  panelIndex: { type: Number, required: true },
  panelRef: { type: String, required: true },
  panelContainerRef: { type: String, required: true },
  initialMaximizedFull: Boolean,
  initialMaximizedContainer: Boolean,
  isVisible: Boolean,
  isMaximizedContainerEnabled: Boolean,
});

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

<style scoped>
.unlocked {
  color: #c6c6c6;
}

.locked {
  color: #e3ddba;
}
</style>
