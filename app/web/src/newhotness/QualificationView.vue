<template>
  <li class="rounded border border-neutral-600 [&>div]:p-xs">
    <div class="border-b border-neutral-600 text-sm">
      {{ qualification.name }}
    </div>

    <div class="flex flex-col gap-xs text-sm">
      <StatusMessageBox
        :status="qualificationStatus ?? 'unknown'"
        class="break-all"
      >
        <template v-if="qualificationStatus === 'success'">
          <div class="w-full flex flex-row gap-xs">
            Passed!
            <IconButton
              v-if="qualification.avId"
              tooltip="Rerun qualification"
              tooltipPlacement="top"
              icon="refresh"
              iconTone="action"
              size="sm"
              class="self-center"
              @click.left="enqueueDVU"
            />
          </div>
        </template>
        <div
          v-else-if="
            qualificationStatus === 'failure' ||
            qualificationStatus === 'warning'
          "
          class="w-full flex flex-row gap-xs"
        >
          <span class="grow">
            {{ qualification.message }}
          </span>
          <button
            v-if="
              (qualification.avId || qualification.output) &&
              (qualification.message || output?.length)
            "
            tabindex="-1"
            class="underline text-action-400 shrink-0"
            @click="toggleHidden"
          >
            {{ showDetails ? "Hide" : "View" }} Details
          </button>
          <IconButton
            v-if="qualification.avId"
            tooltip="Rerun qualification"
            tooltipPlacement="top"
            icon="refresh"
            iconTone="action"
            size="sm"
            class="self-center"
            @click.left="enqueueDVU"
          />
        </div>
        <template v-else>
          The qualification has not yet ran or is actively running.
        </template>
      </StatusMessageBox>

      <template v-if="showDetails">
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
    </div>
  </li>
</template>

<script lang="ts" setup>
import { computed, ref, toRef } from "vue";
import * as _ from "lodash-es";
import { IconButton, LoadingMessage } from "@si/vue-lib/design-system";
import StatusMessageBox from "@/components/StatusMessageBox.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import { Qualification } from "@/newhotness/QualificationPanel.vue";
import { routes, useApi, funcRunTypes } from "@/newhotness/api_composables";

const api = useApi();

const props = defineProps<{
  qualification: Qualification;
  component: string;
}>();

const showDetails = ref(false);

const qualification = toRef(props, "qualification");

const qualificationStatus = computed(() => {
  if (_.isNil(props.qualification.status)) return undefined;
  return props.qualification.status;
});

const dvuApi = useApi();
const enqueueDVU = async () => {
  if (props.qualification.avId) {
    const call = dvuApi.endpoint(routes.EnqueueAttributeValue, {
      id: props.component,
      attributeValue: props.qualification.avId,
    });

    // This route can mutate head, so we do not need to handle new change set semantics.
    await call.post([props.qualification.avId]);
  }
};
const toggleHidden = async () => {
  showDetails.value = !showDetails.value;

  if (!showDetails.value) return;

  if (props.qualification.output) {
    output.value = props.qualification.output;
    return;
  }

  if (props.qualification.avId) {
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
