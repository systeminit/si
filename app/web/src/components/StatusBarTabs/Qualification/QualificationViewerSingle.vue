<template>
  <div
    class="rounded text-left flex flex-col overflow-auto bg-neutral-800 border border-neutral-800 drop-shadow-md"
  >
    <div class="flex py-2 mb-px px-3 capitalize bg-black">
      {{ qualification.title }}
    </div>

    <div class="w-full flex flex-col px-3 py-3 gap-2 text-sm bg-black">
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

      <div class="text-right">
        <button
          class="underline text-action-400"
          @click="detailsModalRef?.open()"
        >
          View Details
        </button>
      </div>
    </div>

    <Modal ref="detailsModalRef" size="2xl" :title="qualification.title">
      <div class="my-2">
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
        class="flex flex-col my-2 p-2 border border-destructive-600 text-destructive-500 rounded"
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
        class="flex flex-col my-2 p-2 border border-warning-600 text-warning-500 rounded"
      >
        <b>Raw Output:</b>
        <p
          v-for="(output, index) in qualification.output"
          :key="index"
          class="text-sm break-all"
        >
          {{ output.line }}
        </p>
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
import { trackEvent } from "@/utils/tracking";

const props = defineProps<{
  qualification: Qualification;
  componentId: string;
}>();

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
      qualification_runs_on_component: props.componentId,
    });
  },
  { immediate: true },
);

const detailsModalRef = ref<InstanceType<typeof Modal>>();

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
</script>
