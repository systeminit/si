<template>
  <div v-if="selectedComponent" class="flex flex-col h-full">
    <!-- <div class="p-xs border-b dark:border-neutral-600">
      <Inline align-y="center">
        <Icon size="md" name="plug" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Component Details</div>
      </Inline>
    </div> -->

    <div v-if="DEV_MODE" class="px-xs pt-xs text-2xs italic opacity-30">
      COMPONENT ID =
      <span @click="openDebugModal(selectedComponent?.id)">{{
        selectedComponent?.id
      }}</span>
      <br />
      NODE ID = {{ selectedComponent.nodeId }}
    </div>
    <ComponentCard :componentId="selectedComponent.id" class="m-xs" />

    <div
      v-if="currentStatus && currentStatus.isUpdating"
      class="flex flex-row items-center gap-xs pl-xs"
    >
      <!-- currently updating -->
      <Icon name="loader" size="lg" class="text-action-500 shrink-0" />
      <div class="grow truncate py-xs">
        {{ currentStatus.statusMessage }}
      </div>
    </div>
    <div v-else class="flex flex-row items-center">
      <DetailsPanelTimestamps
        :changeStatus="selectedComponent.changeStatus"
        :created="selectedComponent.createdInfo"
        :modified="selectedComponent.updatedInfo"
        :deleted="selectedComponent.deletedInfo"
      />
      <div class="pr-xs shrink-0">
        <VButton
          v-if="selectedComponent.resource.data"
          icon="refresh"
          size="sm"
          variant="ghost"
          loadingIcon="refresh-active"
          loadingText="Refreshing..."
          :loading="refreshing"
          @click="onClickRefreshButton"
        >
          Resource
        </VButton>
      </div>
    </div>

    <template v-if="selectedComponent.changeStatus === 'deleted'">
      <Stack v-if="!changeSetStore.headSelected" class="p-sm">
        <ErrorMessage icon="alert-triangle" tone="warning">
          This component will be removed from your model when this change set is
          merged
        </ErrorMessage>
        <VButton
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
        <TabGroup
          :startSelectedTabSlug="
            changeSetStore.headSelected ? 'resource' : 'attributes'
          "
          :rememberSelectedTabKey="`component_details_${
            changeSetStore.headSelected ? 'view' : 'model'
          }`"
          trackingSlug="component_details"
        >
          <TabGroupItem label="Attributes" slug="attributes">
            <AttributeViewer class="dark:text-neutral-50 text-neutral-900" />
          </TabGroupItem>
          <TabGroupItem label="Code" slug="code">
            <template v-if="codeReqStatus.isPending"> Loading code...</template>
            <template v-else-if="codeReqStatus.isError">
              <ErrorMessage :requestStatus="codeReqStatus" />
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
    <ComponentDebugModal ref="debugModalRef" />
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount, ref } from "vue";
import {
  Icon,
  ErrorMessage,
  VButton,
  Stack,
  TabGroup,
  TabGroupItem,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useStatusStore } from "@/store/status.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import AttributeViewer from "@/components/AttributeViewer.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import ComponentCard from "./ComponentCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import ComponentDetailsResource from "./ComponentDetailsResource.vue";
import ComponentDebugModal from "./ComponentDebugModal.vue";

const emit = defineEmits(["delete", "restore"]);

const DEV_MODE = import.meta.env.DEV;

const debugModalRef = ref<InstanceType<typeof ComponentDebugModal>>();
const openDebugModal = (componentId?: string) => {
  if (debugModalRef.value && componentId) {
    debugModalRef.value?.open(componentId);
  }
};

const componentsStore = useComponentsStore();
const changeSetStore = useChangeSetsStore();

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

const refreshing = computed(() => {
  const componentId = selectedComponent.value?.id;
  if (componentId) {
    return componentsStore.refreshingStatus[componentId] ?? false;
  }

  return false;
});

const onClickRefreshButton = () => {
  if (selectedComponent.value) {
    componentsStore.REFRESH_RESOURCE_INFO(selectedComponent.value.id);
  }
};
</script>
