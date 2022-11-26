<template>
  <div class="flex flex-col h-full">
    <div
      class="w-full flex flex-row p-xs border-b dark:border-neutral-600 items-center"
    >
      <Icon
        :name="selectedComponent.icon"
        size="lg"
        :style="{ color: selectedComponent.color }"
        class="shrink-0 m-xs"
      />

      <!-- NOTE - added some padding here to prevent capsize/truncate/overflow issues from cutting off top/bottom of text -->
      <div
        class="flex flex-col grow gap-xs overflow-x-hidden p-xs"
        :style="{ color: selectedComponent.color }"
      >
        <div class="text-lg font-bold capsize">
          <div class="truncate">
            {{ selectedComponent.displayName }}
          </div>
        </div>
        <!-- <Icon name="menu" class="text-neutral-400 shrink-0" />
      <Icon name="cat" class="text-neutral-400 shrink-0" /> -->
        <div class="text-xs italic capsize">
          <div class="truncate">{{ selectedComponent.schemaName }}</div>
        </div>
      </div>
    </div>

    <div v-if="currentStatus" class="border-b dark:border-neutral-600 p-sm">
      <template v-if="currentStatus.isUpdating">
        <div class="flex flex-row items-center gap-xs">
          <Icon name="loader" size="lg" class="text-action-500 shrink-0" />
          <div class="grow truncate py-xs">
            {{ currentStatus.statusMessage }}
          </div>
          <!-- <span class="text-sm">Details</span> -->
        </div>
      </template>
      <template v-else>
        <div class="font-bold capsize">
          {{ currentStatus.statusMessage }}
        </div>
        <div class="text-xs italic text-neutral-400 capsize mt-xs">
          Updated at
          <Timestamp :date="currentStatus.lastStepCompletedAt" size="long" />
          <template v-if="currentStatus.byActor">
            by
            {{
              currentStatus.byActor.type === "user"
                ? currentStatus.byActor.label
                : "system"
            }}
          </template>
        </div>
      </template>
    </div>

    <div class="flex-grow relative">
      <SiTabGroup>
        <template #tabs>
          <SiTabHeader>Attributes</SiTabHeader>
          <SiTabHeader>Code</SiTabHeader>
          <SiTabHeader>Resource</SiTabHeader>
        </template>

        <template #panels>
          <TabPanel class="w-full">
            <AttributeViewer
              class="dark:text-neutral-50 text-neutral-900"
              :disabled="props.disabled"
            />
          </TabPanel>

          <TabPanel class="w-full h-full overflow-hidden">
            <template v-if="codeReqStatus.isPending"> Loading code...</template>
            <template v-else-if="codeReqStatus.isError">
              <ErrorMessage :request-status="codeReqStatus" />
            </template>
            <template
              v-else-if="codeReqStatus.isSuccess && selectedComponentCode"
            >
              <CodeViewer
                :code="
                  selectedComponentCode[0]?.code || '# No code generated yet'
                "
                class="dark:text-neutral-50 text-neutral-900"
              >
                <template #title>
                  <span
                    class="text-lg ml-4 whitespace-nowrap overflow-hidden text-ellipsis"
                    >{{ selectedComponent.displayName }} Code</span
                  >
                </template>
              </CodeViewer>
            </template>
          </TabPanel>

          <TabPanel class="w-full h-full mt-3">
            <CodeViewer
              :code="
                selectedComponent.resource.data
                  ? JSON.stringify(selectedComponent.resource.data, null, 2)
                  : ''
              "
              class="dark:text-neutral-50 text-neutral-900 pt-4"
            >
              <template #title>
                <HealthIcon
                  :health="selectedComponent.resource.status"
                  :message="
                    selectedComponent.resource.message
                      ? [selectedComponent.resource.message]
                      : []
                  "
                  :view-details="selectedComponent.resource.logs"
                  class="ml-3"
                />
              </template>
            </CodeViewer>
          </TabPanel>
        </template>
      </SiTabGroup>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { computed, onBeforeMount } from "vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import AttributeViewer from "@/organisms/AttributeViewer.vue";
import CodeViewer from "@/organisms/CodeViewer.vue";
import HealthIcon from "@/molecules/HealthIcon.vue";
import { useComponentsStore } from "@/store/components.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import { useStatusStore } from "@/store/status.store";
import Timestamp from "@/ui-lib/Timestamp.vue";
import Icon from "@/ui-lib/icons/Icon.vue";

const props = defineProps<{
  disabled?: boolean;
}>();

const componentsStore = useComponentsStore();

const selectedComponent = computed(() => componentsStore.selectedComponent);
const selectedComponentId = computed(() => componentsStore.selectedComponentId);

const selectedComponentCode = computed(
  () => componentsStore.selectedComponentCode,
);

// this component has a :key so a new instance will be re-mounted when the selected component changes
// so we can use mounted hooks to trigger fetching data
onBeforeMount(() => {
  if (selectedComponentId.value) {
    componentsStore.FETCH_COMPONENT_CODE(selectedComponentId.value);
  }
});

const codeReqStatus = componentsStore.getRequestStatus(
  "FETCH_COMPONENT_CODE",
  selectedComponentId,
);

const statusStore = useStatusStore();
const currentStatus = computed(() =>
  selectedComponentId.value
    ? statusStore.componentStatusById[selectedComponentId.value]
    : undefined,
);
</script>
