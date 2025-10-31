<template>
  <li class="rounded border border-neutral-600 [&>div]:p-xs">
    <div
      class="border-b border-neutral-600 text-sm flex flex-row items-center justify-between gap-xs"
    >
      <TruncateWithTooltip :lineClamp="3">
        {{ qualification.name }}
      </TruncateWithTooltip>
      <NewButton
        v-if="qualification.avId"
        label="Rerun qualification"
        @click="enqueueDVU"
      />
    </div>

    <div class="flex flex-col gap-xs text-sm">
      <StatusMessageBox
        :status="qualificationStatus ?? 'unknown'"
        class="break-all"
      >
        <template v-if="qualificationStatus === 'success'"> Passed! </template>
        <div
          v-else-if="
            qualificationStatus === 'failure' ||
            qualificationStatus === 'warning'
          "
          class="w-full flex flex-row items-center gap-xs"
        >
          <span class="grow">
            {{ qualification.message }}
          </span>
          <NewButton
            v-if="
              (qualification.avId || qualification.output) &&
              (qualification.message || output?.length)
            "
            :label="showDetails ? 'Hide Details' : 'View Details'"
            @click="toggleHidden"
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
import {
  LoadingMessage,
  NewButton,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import StatusMessageBox from "@/newhotness/layout_components/StatusMessageBox.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import {
  Qualification,
  QualificationStatus,
} from "@/newhotness/QualificationPanel.vue";
import { routes, useApi, funcRunTypes } from "@/newhotness/api_composables";

const api = useApi();

const props = defineProps<{
  qualification: Qualification;
  component: string;
}>();

const showDetails = ref(false);

const qualification = toRef(props, "qualification");

const qualificationStatus = computed((): QualificationStatus | undefined => {
  return props.qualification.status ?? undefined;
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
