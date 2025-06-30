<template>
  <li class="rounded border border-neutral-600 [&>div]:p-xs">
    <div class="border-b border-neutral-600">
      {{ qualification.name }}
    </div>

    <div class="flex flex-col gap-xs text-sm">
      <StatusMessageBox
        :status="qualificationStatus ?? 'unknown'"
        class="break-all"
      >
        <template v-if="qualificationStatus === 'success'"> Passed! </template>
        <template
          v-else-if="
            qualificationStatus === 'failure' ||
            qualificationStatus === 'warning'
          "
        >
          {{ qualification.message }}
        </template>
        <template v-else>
          The qualification has not yet ran or is actively running.
        </template>
      </StatusMessageBox>

      <template v-if="showDetails && qualification.avId">
        <div
          v-if="output?.length"
          class="my-xs p-xs border border-warning-600 text-warning-500 rounded"
        >
          <b>Raw Output:</b>

          <CodeViewer
            :code="output.map((o) => o).join('\n')"
            :showTitle="false"
            :allowCopy="false"
          >
          </CodeViewer>
        </div>
        <LoadingMessage v-else message="Loading Details..." />
      </template>

      <div
        v-if="qualification.avId && (qualification.message || output?.length)"
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
  </li>
</template>

<script lang="ts" setup>
import { computed, ref, toRef } from "vue";
import * as _ from "lodash-es";
import { LoadingMessage } from "@si/vue-lib/design-system";
import StatusMessageBox from "@/components/StatusMessageBox.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import { Qualification } from "@/newhotness/QualificationPanel.vue";
import { routes, useApi, funcRunTypes } from "@/newhotness/api_composables";

const api = useApi();

const props = defineProps<{
  qualification: Qualification;
}>();

const showDetails = ref(false);

const qualification = toRef(props, "qualification");

const qualificationStatus = computed(() => {
  if (_.isNil(props.qualification.status)) return undefined;
  return props.qualification.status;
});

const toggleHidden = async () => {
  showDetails.value = !showDetails.value;
  if (showDetails.value && props.qualification.avId) {
    const call = api.endpoint<funcRunTypes.FuncRunLogsResponse>(
      routes.FuncRunByAv,
      {
        id: props.qualification.avId,
      },
    );
    const result = await call.get();
    if (api.ok(result)) {
      output.value = result.data.logs.logs.map((l) => l.message);
    }
  }
};

const output = ref<string[] | undefined>(undefined);
</script>
