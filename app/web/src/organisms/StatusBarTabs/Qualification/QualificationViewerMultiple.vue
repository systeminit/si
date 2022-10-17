<template>
  <div class="h-full flex-grow flex flex-col bg-shade-100 min-w-0">
    <div class="overflow-y-auto flex flex-row mt-4 mx-2 flex-wrap">
      <!-- Note(victor): The only reason there's this extra Div here is to allow us to have margins between -->
      <!-- QualificationViews while using flex-basis to keep stuff responsive. We should revisit this and tune -->
      <!-- the breakpoints after the content and design of the View is solidified -->

      <div v-if="qualificationsReqStatus.isPending">Loading...</div>
      <template v-else>
        <div
          v-for="(qualification, index) in componentQualifications"
          :key="index"
          class="basis-full lg:basis-1/2 xl:basis-1/3 overflow-hidden pb-4 px-2"
        >
          <QualificationViewerSingle :qualification="qualification" />
        </div>
      </template>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, watch } from "vue";
import QualificationViewerSingle from "@/organisms/StatusBarTabs/Qualification/QualificationViewerSingle.vue";
import { useQualificationsStore } from "@/store/qualifications.store";
import { useChangeSetsStore } from "@/store/change_sets.store";

const props = defineProps<{
  componentId: number;
}>();

const changeSetsStore = useChangeSetsStore();

const qualificationsStore = useQualificationsStore();
const qualificationsReqStatus = qualificationsStore.getRequestStatus(
  "FETCH_COMPONENT_QUALIFICATIONS",
  computed(() => props.componentId),
);

watch(
  [() => props.componentId, changeSetsStore.selectedChangeSetWritten],
  () => {
    qualificationsStore.FETCH_COMPONENT_QUALIFICATIONS(props.componentId);
  },
  { immediate: true },
);

const componentQualifications = computed(
  () => qualificationsStore.qualificationsByComponentId[props.componentId],
);
</script>
