<template>
  <div>
    <template v-if="resourceReqStatus.isPending"> Loading resource...</template>
    <template v-else-if="resourceReqStatus.isError">
      <ErrorMessage :requestStatus="resourceReqStatus" />
    </template>
    <template v-else-if="resourceReqStatus.isSuccess && selectedComponentResource">
      <CodeViewer
        v-if="selectedComponentResource.payload !== null"
        :code="selectedComponentResource.payload ? JSON.stringify(selectedComponentResource.payload, null, 2) : ''"
        class="dark:text-neutral-50 text-neutral-900 pt-4"
      >
        <template #title>
          <StatusIndicatorIcon type="resource" :status="selectedComponentResource.status" />
          <div class="pl-sm grow overflow-hidden">
            <div class="font-bold line-clamp-2 break-all">
              {{
                selectedComponentResource.message
                  ? selectedComponentResource.message
                  : `Health ${selectedComponentResource.status}`
              }}
            </div>
            <div v-if="selectedComponentResource.lastSynced" class="text-xs italic truncate">
              Last synced:
              <Timestamp :date="new Date(selectedComponentResource.lastSynced)" size="long" />
            </div>
          </div>
          <div class="pr-sm">
            <ActionRunnerDetails
              :health="selectedComponentResource.status"
              :message="[selectedComponentResource.message ?? ''].filter((f) => f.length > 0)"
            />
          </div>
        </template>
      </CodeViewer>
      <div v-else class="flex flex-col items-center p-sm">
        <div class="w-64"><EmptyStateIcon name="no-changes" /></div>
        <div class="w-full text-center text-xl text-neutral-400">
          This component does not have a resource associated with it
        </div>
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed, watch } from "vue";
import { ErrorMessage, Timestamp } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useViewsStore } from "@/store/views.store";
import CodeViewer from "./CodeViewer.vue";
import ActionRunnerDetails from "./Actions/ActionRunnerDetails.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import EmptyStateIcon from "./EmptyStateIcon.vue";

const changeSetsStore = useChangeSetsStore();
const viewStore = useViewsStore();
const componentsStore = useComponentsStore();
const selectedComponentId = computed(() => viewStore.selectedComponentId!);

const resourceReqStatus = componentsStore.getRequestStatus("FETCH_COMPONENT_RESOURCE", selectedComponentId);

const selectedComponentResource = computed(
  () => componentsStore.componentResourceById[viewStore.selectedComponentId || ""],
);

watch(
  [() => changeSetsStore.selectedChangeSetLastWrittenAt],
  () => {
    if (
      viewStore.selectedComponent &&
      "changeStatus" in viewStore.selectedComponent.def &&
      viewStore.selectedComponent.def.changeStatus !== "deleted"
    ) {
      componentsStore.FETCH_COMPONENT_RESOURCE(selectedComponentId.value);
    }
  },
  { immediate: true },
);
</script>
