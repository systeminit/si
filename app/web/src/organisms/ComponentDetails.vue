<template>
  <SiTabGroup>
    <template #tabs>
      <SiTabHeader>Attributes</SiTabHeader>
      <SiTabHeader>Code</SiTabHeader>
      <SiTabHeader>Resource</SiTabHeader>
    </template>

    <template #panels>
      <TabPanel class="w-full">
        <!-- FIXME(nick): remove AttributeViewer's requirement of a componentId -->
        <AttributeViewer class="dark:text-neutral-50 text-neutral-900" />
      </TabPanel>

      <TabPanel class="w-full h-full overflow-hidden">
        <template v-if="codeReqStatus.isPending"> Loading code...</template>
        <template v-else-if="codeReqStatus.isError">
          <ErrorMessage :request-status="codeReqStatus" />
        </template>
        <template v-else-if="codeReqStatus.isSuccess && selectedComponentCode">
          <CodeViewer
            :code="selectedComponentCode[0]?.code || '# No code generated yet'"
            class="dark:text-neutral-50 text-neutral-900"
          >
            <template #title>
              <span
                class="text-lg ml-4 whitespace-nowrap overflow-hidden text-ellipsis"
                >{{ selectedComponent.displayName }} Code</span
              >
            </template>

            <template #actionButtons>
              <SiButtonIcon
                tooltip-text="Re-generate code"
                ignore-text-color
                class="mr-4"
                :icon="isCodeSyncing ? 'refresh-active' : 'refresh'"
                @click="triggerCodeGen"
              />
            </template>
          </CodeViewer>
        </template>
      </TabPanel>

      <TabPanel class="w-full">
        <SiCollapsible
          v-if="selectedComponent.resource"
          text-size="md"
          show-label-and-slot
        >
          <!--<template #label>
            <HealthIcon
              :health="selectedComponent.resource.health"
              size="md"
              hide-text
            />
          </template>-->
          <div class="px-xs pb-xs max-h-96 overflow-hidden flex">
            <div class="flex-grow">
              <CodeViewer
                :code="JSON.stringify(selectedComponent.resource.data)"
                border
              >
                <!--<template #title>
                  <HealthIcon :health="selectedComponent.resource.health" />
                </template>-->
              </CodeViewer>
            </div>
          </div>
        </SiCollapsible>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { computed, onBeforeMount, ref, watch } from "vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import AttributeViewer from "@/organisms/AttributeViewer.vue";
import CodeViewer from "@/organisms/CodeViewer.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import HealthIcon from "@/molecules/HealthIcon.vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import { useComponentsStore } from "@/store/components.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";

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

// we track this here since the update flow is a little weird
// we may wnat to change the trigger code gen endpoint to just return the new code directly
const isCodeSyncing = ref(false);
watch(codeReqStatus, () => {
  // stop spinner when no longer pending...
  if (!codeReqStatus.value.isPending) isCodeSyncing.value = false;
});

function triggerCodeGen() {
  if (!selectedComponentId.value) return;
  isCodeSyncing.value = true;
  componentsStore.TRIGGER_COMPONENT_CODE_GEN(selectedComponentId.value);
}
</script>
