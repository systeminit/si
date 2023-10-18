<template>
  <div class="p-xs overflow-auto">
    <div
      v-for="(qualification, index) in componentQualifications"
      :key="index"
      class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-xs"
    >
      <QualificationViewerSingle
        :qualification="qualification"
        :componentId="props.componentId"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import * as _ from "lodash-es";

import { ComponentId } from "@/store/components.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import QualificationViewerSingle from "@/components/QualificationViewerSingle.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const qualificationsStore = useQualificationsStore();

const componentQualifications = computed(
  () => qualificationsStore.qualificationsByComponentId[props.componentId],
);
</script>
