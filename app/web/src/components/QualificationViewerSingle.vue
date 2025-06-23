<template>
  <div
    class="rounded text-left flex flex-col overflow-auto border border-neutral-600"
  >
    <div class="flex p-xs capitalize border-b border-neutral-600">
      {{ qualification.title }}
    </div>

    <div class="w-full flex flex-col p-xs gap-xs text-sm">
      <StatusMessageBox :status="qualificationStatus" class="break-all">
        <span
          v-if="qualificationStatus === 'failure'"
          :title="titleFailedSubchecks"
        >
          {{ truncatedFailedSubchecks }}
        </span>
        <template v-else-if="qualificationStatus === 'warning'">
          {{ truncatedFailedSubchecks }}
        </template>
        <template v-else-if="qualificationStatus === 'success'">
          Passed!
        </template>
        <template v-else> Qualification running, standby...</template>
      </StatusMessageBox>

      <div v-if="qualification.description">
        <b>Description: </b>
        <p>{{ qualification.description }}</p>
      </div>

      <div v-if="showDetails && !displayDetailsInModal">
        <div
          v-if="failedSubchecks.length"
          class="flex flex-col my-2 p-xs border border-destructive-600 text-destructive-500 rounded"
        >
          <b>Qualification Failures:</b>
          <ul>
            <li
              v-for="(subCheck, idx) in failedSubchecks"
              :key="idx"
              class="p-2 break-words"
            >
              <CodeViewer
                :allowCopy="false"
                :code="subCheck.description"
                :showTitle="false"
              >
              </CodeViewer>
            </li>
          </ul>
        </div>

        <div
          v-if="qualification.output?.length"
          class="flex flex-col my-xs p-xs border border-warning-600 text-warning-500 rounded"
        >
          <b>Raw Output:</b>

          <CodeViewer
            :code="qualification.output.map((o) => o.line).join('\n')"
            :showTitle="false"
            :allowCopy="false"
          >
          </CodeViewer>
        </div>
      </div>

      <div
        v-if="
          qualification.description ||
          failedSubchecks.length ||
          qualification.output?.length
        "
        class="text-right"
      >
        <button
          tabindex="-1"
          class="underline text-action-400"
          @click="toggleHidden"
        >
          {{ showDetails ? "Hide" : "View" }} Details
        </button>
      </div>
    </div>

    <Modal ref="detailsModalRef" :title="qualification.title" size="2xl">
      <div class="my-xs">
        <StatusMessageBox :status="qualificationStatus">
          <template v-if="qualificationStatus === 'failure'">
            Something went wrong!
          </template>
          <template v-else-if="qualificationStatus === 'success'">
            Passed!
          </template>
          <template v-else> Qualification running, standby...</template>
        </StatusMessageBox>
      </div>

      <div v-if="qualification.description" class="my-2">
        <b>Description: </b>
        <p>{{ qualification.description }}</p>
      </div>

      <div
        v-if="failedSubchecks.length"
        class="flex flex-col my-2 p-xs border border-destructive-600 text-destructive-500 rounded"
      >
        <b>Qualification Failures:</b>
        <ul>
          <li v-for="(subCheck, idx) in failedSubchecks" :key="idx" class="p-2">
            {{ subCheck.description }}
          </li>
        </ul>
      </div>

      <div
        v-if="qualification.output?.length"
        class="flex flex-col my-xs p-xs border border-warning-600 text-warning-500 rounded"
      >
        <b>Raw Output:</b>
        <CodeViewer
          :code="qualification.output.map((o) => o.line).join('\n')"
          :showTitle="false"
          :allowCopy="false"
        >
        </CodeViewer>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, toRef, watch } from "vue";
import * as _ from "lodash-es";
import { Modal } from "@si/vue-lib/design-system";
import { Qualification } from "@/api/sdf/dal/qualification";
import StatusMessageBox from "@/components/StatusMessageBox.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import { trackEvent } from "@/utils/tracking";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "./ModelingDiagram/diagram_types";

const props = defineProps<{
  qualification: Qualification;
  component: DiagramNodeData | DiagramGroupData;
  displayDetailsInModal?: boolean;
}>();

const showDetails = ref(false);
const detailsModalRef = ref();

const qualification = toRef(props, "qualification");

const failedSubchecks = computed(
  () =>
    qualification.value.result?.sub_checks.filter(
      (subCheck) =>
        subCheck.status === "failure" || subCheck.status === "warning",
    ) ?? [],
);

const qualificationStatus = computed(() => {
  if (_.isNil(props.qualification.result)) return "running";
  return props.qualification.result.status;
});

// Let's create an event if the qualification status changes
// this means we can understand when a qualification has changed
watch(
  qualificationStatus,
  () => {
    trackEvent("qualification_status", {
      qualification_name: qualification.value.title,
      qualification_status: qualification.value.result?.status,
      qualification_runs_on_component: props.component.def.id,
    });
  },
  { immediate: true },
);

const titleFailedSubchecks = computed(() => {
  return failedSubchecks.value.length
    ? failedSubchecks.value.map((c) => c.description).join(" ")
    : 'Something went wrong! Click "View Details" to see the output.';
});

const truncatedFailedSubchecks = computed(() => {
  const maxLength = 100;
  const message = failedSubchecks.value.length
    ? failedSubchecks.value.map((c) => c.description).join(" ")
    : 'Something went wrong! Click "View Details" to see the output.';
  if (message.length <= maxLength + 3) return message;
  return `${message.slice(0, maxLength)}...`;
});

const toggleHidden = () => {
  if (props.displayDetailsInModal) {
    detailsModalRef.value.open();
  } else {
    showDetails.value = !showDetails.value;
  }
};
</script>
