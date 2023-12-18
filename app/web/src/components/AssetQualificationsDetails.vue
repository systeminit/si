<template>
  <div class="p-xs overflow-auto">
    <template v-if="qualificationDetailsReqStatus.isFirstLoad">
      Loading...
    </template>

    <template v-else-if="qualificationDetailsReqStatus.isError">
      <ErrorMessage :requestStatus="qualificationDetailsReqStatus" />
    </template>
    <template v-else>
      <div
        v-for="(qualification, index) in componentQualificationsSorted"
        :key="index"
        class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-xs"
      >
        <QualificationViewerSingle
          :qualification="qualification"
          :componentId="props.componentId"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, PropType, watch } from "vue";
import * as _ from "lodash-es";

import { ErrorMessage } from "@si/vue-lib/design-system";
import { ComponentId } from "@/store/components.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import QualificationViewerSingle from "@/components/QualificationViewerSingle.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const changeSetsStore = useChangeSetsStore();
const qualificationsStore = useQualificationsStore();

const componentQualifications = computed(
  () => qualificationsStore.qualificationsByComponentId[props.componentId],
);

const componentQualificationsSorted = computed(() =>
  _.sortBy(componentQualifications.value, "title"),
);

const qualificationDetailsReqStatus = qualificationsStore.getRequestStatus(
  "FETCH_COMPONENT_QUALIFICATIONS",
  props.componentId,
);

// TODO: this logic probably shouldnt live here... and more targeted updates should be sent
watch(
  [() => changeSetsStore.selectedChangeSetLastWrittenAt],
  () => {
    qualificationsStore.FETCH_COMPONENT_QUALIFICATIONS(props.componentId);
  },
  { immediate: true },
);
</script>
