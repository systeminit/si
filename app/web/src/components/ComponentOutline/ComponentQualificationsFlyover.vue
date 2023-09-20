<template>
  <div
    :class="
      clsx(
        'bg-neutral-700 w-96 h-80 rounded flex flex-col overflow-clip text-white shadow-3xl dark',
        themeContainerClasses,
      )
    "
  >
    <div
      class="bg-black uppercase font-bold text-md p-xs flex place-content-between items-center"
    >
      <span>Qualifications - {{ component?.displayName }}</span>
      <div class="flex gap-xs p-2xs">
        <span v-if="qualificationsFailed" class="flex items-center gap-0.5">
          <StatusIndicatorIcon
            class="inline-block"
            type="qualification"
            status="failure"
            size="md"
          />
          {{ qualificationsFailed }}
        </span>
        <span v-if="qualificationsWarned" class="flex items-center gap-0.5">
          <StatusIndicatorIcon
            class="inline-block"
            type="qualification"
            status="warning"
            size="md"
          />
          {{ qualificationsWarned }}
        </span>
        <span class="flex items-center gap-0.5">
          <StatusIndicatorIcon
            class="inline-block"
            type="qualification"
            status="success"
            size="md"
          />
          {{ qualificationsSucceeded }}
        </span>
      </div>
    </div>
    <div class="p-xs pb-0 overflow-auto">
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
  </div>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import * as _ from "lodash-es";

import { useThemeContainer } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import StatusIndicatorIcon from "../StatusIndicatorIcon.vue";
import QualificationViewerSingle from "../StatusBarTabs/Qualification/QualificationViewerSingle.vue";

const { themeContainerClasses } = useThemeContainer("dark");

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();

const component = computed(
  () => componentsStore.componentsById[props.componentId],
);

const componentQualifications = computed(
  () => qualificationsStore.qualificationsByComponentId[props.componentId],
);

const qualificationStats = computed(
  () => qualificationsStore.qualificationStatsByComponentId[props.componentId],
);
const qualificationsFailed = computed(() =>
  qualificationStats.value ? qualificationStats.value.failed : 0,
);
const qualificationsWarned = computed(() =>
  qualificationStats.value ? qualificationStats.value.warned : 0,
);
const qualificationsSucceeded = computed(() =>
  qualificationStats.value ? qualificationStats.value.succeeded : 0,
);
</script>
