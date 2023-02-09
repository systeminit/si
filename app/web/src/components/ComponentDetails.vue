<template>
  <div class="flex flex-col h-full">
    <!-- <div class="p-xs border-b dark:border-neutral-600">
      <Inline align-y="center">
        <Icon size="md" name="plug" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Component Details</div>
      </Inline>
    </div> -->

    <ComponentCard :component-id="selectedComponent.id" class="m-xs" />

    <div class="m-xs mt-0 text-xs italic text-neutral-300">
      <Inline
        spacing="2xs"
        :class="
          clsx(selectedComponent.changeStatus === 'added' && 'text-success-500')
        "
      >
        <Icon name="plus-circle" size="xs" />
        {{ formatters.timeAgo(selectedComponent.createdInfo.timestamp) }} by
        {{ selectedComponent.createdInfo.actor.label }}
      </Inline>
      <Inline
        v-if="
          (selectedComponent.changeStatus === 'modified' ||
            selectedComponent.changeStatus === 'unmodified') &&
          selectedComponent.createdInfo.timestamp !==
            selectedComponent.updatedInfo.timestamp
        "
        spacing="2xs"
        :class="
          clsx(
            selectedComponent.changeStatus === 'modified' && 'text-warning-500',
          )
        "
      >
        <Icon name="tilde-circle" size="xs" />
        {{ formatters.timeAgo(selectedComponent.updatedInfo.timestamp) }} by
        {{ selectedComponent.updatedInfo.actor.label }}
      </Inline>
      <Inline
        v-if="selectedComponent.changeStatus === 'deleted'"
        class="text-destructive-500"
        spacing="2xs"
      >
        <Icon name="minus-circle" size="xs" />
        <!-- {{ formatters.timeAgo(selectedComponent.updatedInfo.timestamp) }} by
        {{ selectedComponent.updatedInfo.actor.label }} -->
        {{ formatters.timeAgo(selectedComponent.deletedAt) }} by
        {{ selectedComponent.createdInfo.actor.label }}
      </Inline>
    </div>

    <div
      v-if="currentStatus"
      class="border-b dark:border-neutral-600 border-t p-sm"
    >
      <template v-if="currentStatus.isUpdating">
        <!-- currently updating -->
        <div class="flex flex-row items-center gap-xs">
          <Icon name="loader" size="lg" class="text-action-500 shrink-0" />
          <div class="grow truncate py-xs">
            {{ currentStatus.statusMessage }}
          </div>
          <!-- <span class="text-sm">Details</span> -->
        </div>
      </template>
      <template v-else>
        <!-- not currently updating -->
        <div class="font-bold capsize">
          {{ currentStatus.statusMessage }}
        </div>
        <div class="text-xs italic text-neutral-400 capsize mt-xs">
          Updated at
          <Timestamp :date="new Date(currentStatus.lastUpdateAt)" size="long" />
          <template v-if="currentStatus.lastUpdateBy">
            by
            {{ currentStatus.lastUpdateBy.label }}
          </template>
        </div>
      </template>
    </div>

    <template v-if="selectedComponent.changeStatus === 'deleted'">
      <p>DELETED!</p>
      <VButton2 />
    </template>
    <template v-else>
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
  </template>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { TabPanel } from "@headlessui/vue";
import { computed, onBeforeMount } from "vue";
import { useComponentsStore } from "@/store/components.store";
import { useStatusStore } from "@/store/status.store";
import SiTabGroup from "@/components/SiTabGroup.vue";
import SiTabHeader from "@/components/SiTabHeader.vue";
import AttributeViewer from "@/components/AttributeViewer.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import HealthIcon from "@/components/HealthIcon.vue";
import Timestamp from "@/ui-lib/Timestamp.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import Inline from "@/ui-lib/layout/Inline.vue";
import formatters from "@/ui-lib/helpers/formatting";
import ComponentCard from "./ComponentCard.vue";

const props = defineProps<{
  disabled?: boolean;
}>();

const emit = defineEmits(["delete", "restore"]);

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
