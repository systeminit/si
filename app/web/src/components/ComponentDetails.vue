<template>
  <div class="flex flex-col h-full">
    <!-- <div class="p-xs border-b dark:border-neutral-600">
      <Inline align-y="center">
        <Icon size="md" name="plug" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Component Details</div>
      </Inline>
    </div> -->

    <div v-if="DEV_MODE" class="px-xs pt-xs text-2xs italic opacity-30">
      COMPONENT ID = {{ selectedComponent.id }}
      <br />
      NODE ID = {{ selectedComponent.nodeId }}
    </div>
    <ComponentCard :component-id="selectedComponent.id" class="m-xs" />
    <template v-if="currentStatus && currentStatus.isUpdating">
      <!-- currently updating -->
      <div class="flex flex-row items-center gap-xs pl-xs">
        <Icon name="loader" size="lg" class="text-action-500 shrink-0" />
        <div class="grow truncate py-xs">
          {{ currentStatus.statusMessage }}
        </div>
        <!-- <span class="text-sm">Details</span> -->
      </div>
    </template>
    <DetailsPanelTimestamps
      v-else
      :change-status="selectedComponent.changeStatus"
      :created="selectedComponent.createdInfo"
      :modified="selectedComponent.updatedInfo"
      :deleted="selectedComponent.deletedInfo"
    />

    <template v-if="selectedComponent.changeStatus === 'deleted'">
      <Stack v-if="!props.disabled" class="p-sm">
        <ErrorMessage icon="alert-triangle" tone="warning">
          This component will be removed from your model when this change set is
          merged
        </ErrorMessage>
        <VButton2
          tone="shade"
          variant="ghost"
          size="md"
          icon="trash-restore"
          :label="`Restore ${
            selectedComponent.nodeType === 'component' ? 'Component' : 'Frame'
          }`"
          @click="emit('restore')"
        />
      </Stack>
    </template>

    <template v-else>
      <div class="flex-grow relative">
        <TabGroup start-selected-tab-slug="attributes">
          <TabGroupItem label="Attributes" slug="attributes">
            <AttributeViewer
              class="dark:text-neutral-50 text-neutral-900"
              :disabled="props.disabled"
            />
          </TabGroupItem>
          <TabGroupItem label="Code" slug="code">
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
                class="dark:text-neutral-50 text-neutral-900 pt-2"
              >
                <template #title>
                  <div
                    class="text-lg ml-4 whitespace-nowrap overflow-hidden text-ellipsis"
                  >
                    {{ selectedComponent.displayName }} Code
                  </div>
                </template>
              </CodeViewer>
            </template>
          </TabGroupItem>
          <TabGroupItem label="Resource" slug="resource">
            <ComponentDetailsResource />
          </TabGroupItem>
        </TabGroup>
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount } from "vue";
import { useComponentsStore } from "@/store/components.store";
import { useStatusStore } from "@/store/status.store";
import AttributeViewer from "@/components/AttributeViewer.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import TabGroup from "@/ui-lib/tabs/TabGroup.vue";
import TabGroupItem from "@/ui-lib/tabs/TabGroupItem.vue";
import ComponentCard from "./ComponentCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import ComponentDetailsResource from "./ComponentDetailsResource.vue";

const props = defineProps<{
  disabled?: boolean;
}>();

const emit = defineEmits(["delete", "restore"]);

const DEV_MODE = import.meta.env.DEV;

const componentsStore = useComponentsStore();

const selectedComponent = computed(() => componentsStore.selectedComponent);
const selectedComponentId = computed(() => componentsStore.selectedComponentId);

const selectedComponentCode = computed(
  () => componentsStore.selectedComponentCode,
);

// this component has a :key so a new instance will be re-mounted when the selected component changes
// so we can use mounted hooks to trigger fetching data
onBeforeMount(() => {
  if (
    selectedComponentId.value &&
    selectedComponent.value?.changeStatus !== "deleted"
  ) {
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
