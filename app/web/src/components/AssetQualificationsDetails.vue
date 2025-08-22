<template>
  <div class="p-xs overflow-auto">
    <div
      v-for="(qualification, index) in componentQualificationsSorted"
      :key="index"
      class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-xs"
    >
      <QualificationViewerSingle
        :qualification="qualification"
        :component="props.component"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import * as _ from "lodash-es";

import { useQualificationsStore } from "@/store/qualifications.store";
import QualificationViewerSingle from "@/components/QualificationViewerSingle.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "./ModelingDiagram/diagram_types";

const props = defineProps<{
  component: DiagramGroupData | DiagramNodeData;
}>();

const qualificationsStore = useQualificationsStore();

const componentQualifications = computed(
  () => qualificationsStore.qualificationsByComponentId[props.component.def.id],
);

const componentQualificationsSorted = computed(() =>
  _.sortBy(componentQualifications.value, "title"),
);
</script>
