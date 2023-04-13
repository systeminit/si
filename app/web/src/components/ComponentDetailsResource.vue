<template>
  <div v-if="selectedComponent">
    <CodeViewer
      v-if="selectedComponent.resource.data !== null"
      :code="
        selectedComponent.resource.data
          ? JSON.stringify(selectedComponent.resource.data, null, 2)
          : ''
      "
      class="dark:text-neutral-50 text-neutral-900 pt-4"
    >
      <template #title>
        <StatusIndicatorIcon
          type="resource"
          :status="selectedComponent.resource.status"
        />
        <div class="pl-sm grow overflow-hidden">
          <div class="font-bold line-clamp-2 break-all">
            {{
              selectedComponent.resource.message
                ? selectedComponent.resource.message
                : `Health ${selectedComponent.resource.status}`
            }}
          </div>
          <div
            v-if="selectedComponent.resource.lastSynced"
            class="text-xs italic truncate"
          >
            Last synced:
            <Timestamp
              :date="new Date(selectedComponent.resource.lastSynced)"
              size="long"
            />
          </div>
        </div>
        <div class="pr-sm">
          <FixDetails
            v-if="
              selectedComponent.resource.logs &&
              selectedComponent.resource.logs.length > 0
            "
            :health="selectedComponent.resource.status"
            :message="
              [selectedComponent.resource.message ?? ''].filter(
                (f) => f.length > 0,
              )
            "
            :details="selectedComponent.resource.logs"
          />
        </div>
      </template>
    </CodeViewer>
    <div
      v-else
      class="w-full text-center text-lg mt-5 dark:text-neutral-50 text-neutral-900"
    >
      This component does not have a resource associated with it yet
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { Timestamp } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import CodeViewer from "./CodeViewer.vue";
import FixDetails from "./FixDetails.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const componentsStore = useComponentsStore();
const selectedComponent = computed(() => componentsStore.selectedComponent);
</script>
