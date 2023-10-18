<template>
  <ScrollArea v-if="selectedComponent">
    <template #top>
      <SidebarSubpanelTitle label="Asset Details" icon="component" />

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
    </template>

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
      <div class="absolute inset-0">
        <TabGroup>
          <TabGroupItem>
            <template #label>
              <Inline>
                <span>Component</span>
                <StatusIndicatorIcon
                  v-if="selectedComponentQualificationStatus"
                  type="qualification"
                  :status="selectedComponentQualificationStatus"
                />
              </Inline>
            </template>

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
                <AttributeViewer
                  class="dark:text-neutral-50 text-neutral-900"
                />
              </TabGroupItem>
              <TabGroupItem label="Code" slug="code">
                <template v-if="codeReqStatus.isPending">
                  Loading code...</template
                >
                <template v-else-if="codeReqStatus.isError">
                  <ErrorMessage :requestStatus="codeReqStatus" />
                </template>
                <template
                  v-else-if="codeReqStatus.isSuccess && selectedComponentCode"
                >
                  <div class="absolute inset-xs">
                    <CodeViewer
                      :code="
                        selectedComponentCode[0]?.code ||
                        '# No code generated yet'
                      "
                    >
                    </CodeViewer>
                  </div>
                </template>
              </TabGroupItem>
              <TabGroupItem label="Qualifications" slug="qualifications">
                <AssetQualificationsDetails
                  :componentId="selectedComponentId"
                />
              </TabGroupItem>

              <TabGroupItem label="Diff" slug="diff">
                <AssetDiffDetails :componentId="selectedComponentId" />
              </TabGroupItem>
            </TabGroup>
          </TabGroupItem>
          <TabGroupItem>
            <template #label>
              <Inline>
                <span>Resource</span>
                <StatusIndicatorIcon
                  v-if="selectedComponent.resource.data"
                  type="resource"
                  status="exists"
                />
              </Inline>
            </template>
            <ComponentDetailsResource />
          </TabGroupItem>
          <TabGroupItem>
            <template #label>
              <Inline>
                <span>Actions</span>
                <PillCounter :count="selectedComponentActionsCount" />
              </Inline>
            </template>
            <AssetActionsDetails :componentId="selectedComponentId" />
          </TabGroupItem>
        </TabGroup>
      </div>
    </template>
    <ComponentDebugModal ref="debugModalRef" />
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeMount, ref } from "vue";
import {
  Icon,
  ErrorMessage,
  VButton,
  Stack,
  TabGroup,
  TabGroupItem,
  Inline,
  ScrollArea,
  PillCounter,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useStatusStore } from "@/store/status.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import AttributeViewer from "@/components/AttributeViewer.vue";
import CodeViewer from "@/components/CodeViewer.vue";
import { useQualificationsStore } from "@/store/qualifications.store";
import { useActionsStore } from "@/store/actions.store";
import ComponentCard from "./ComponentCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import ComponentDetailsResource from "./ComponentDetailsResource.vue";
import ComponentDebugModal from "./ComponentDebugModal.vue";
import AssetQualificationsDetails from "./AssetQualificationsDetails.vue";
import AssetActionsDetails from "./AssetActionsDetails.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import AssetDiffDetails from "./AssetDiffDetails.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const emit = defineEmits(["delete", "restore"]);

const DEV_MODE = import.meta.env.DEV;

const debugModalRef = ref<InstanceType<typeof ComponentDebugModal>>();
const openDebugModal = (componentId?: string) => {
  if (debugModalRef.value && componentId) {
    debugModalRef.value?.open(componentId);
  }
};

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();
const changeSetStore = useChangeSetsStore();
const actionsStore = useActionsStore();

const selectedComponent = computed(() => componentsStore.selectedComponent);
const selectedComponentId = computed(
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  () => componentsStore.selectedComponentId!,
);

const selectedComponentQualificationStatus = computed(
  () =>
    qualificationsStore.qualificationStatusByComponentId[
      selectedComponentId.value
    ],
);

const selectedComponentCode = computed(
  () => componentsStore.selectedComponentCode,
);

const selectedComponentActionsCount = computed(() => {
  return _.filter(
    actionsStore.actionsByComponentId[selectedComponentId.value],
    (a) => !!a.actionInstanceId,
  ).length;
});

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
