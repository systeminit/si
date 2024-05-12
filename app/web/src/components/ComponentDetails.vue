<template>
  <ScrollArea v-if="selectedComponent">
    <template #top>
      <ComponentCard :componentId="selectedComponent.id" titleCard>
        <DetailsPanelMenuIcon
          @click="
            (e) => {
              emit('openMenu', e);
            }
          "
        />
      </ComponentCard>
      <div
        v-if="currentStatus && currentStatus.isUpdating"
        class="flex flex-row items-center gap-xs m-xs mt-0"
      >
        <!-- currently updating -->
        <Icon name="loader" size="xs" class="text-action-500 shrink-0" />
        <div class="grow truncate text-xs italic">
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
            v-if="
              selectedComponent.hasResource &&
              changeSetStore.selectedChangeSetId ===
                changeSetStore.headChangeSetId
            "
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
            selectedComponent.componentType === ComponentType.Component
              ? 'Component'
              : 'Frame'
          }`"
          @click="modelingEventBus.emit('restoreSelection')"
        />
      </Stack>
    </template>

    <template v-else>
      <div class="absolute inset-0 border-t dark:border-neutral-700">
        <TabGroup
          ref="tabsRef"
          trackingSlug="asset_details"
          variant="fullsize"
          :startSelectedTabSlug="
            componentsStore.detailsTabSlugs[0] || undefined
          "
          @update:selectedTab="onTabSelected"
        >
          <TabGroupItem slug="component">
            <template #label>
              <Inline noWrap alignY="center">
                <span>Component</span>
                <StatusIndicatorIcon
                  v-if="selectedComponentQualificationStatus"
                  type="qualification"
                  :status="selectedComponentQualificationStatus"
                  size="sm"
                />
              </Inline>
            </template>
            <TabGroup
              ref="componentSubTabsRef"
              trackingSlug="asset_details/component"
              variant="minimal"
              :startSelectedTabSlug="
                componentsStore.detailsTabSlugs[1] || undefined
              "
              marginTop="2xs"
              @update:selectedTab="onTabSelected"
            >
              <TabGroupItem label="Attributes" slug="attributes">
                <AttributesPanel />
              </TabGroupItem>
              <TabGroupItem label="Code" slug="code">
                <ComponentDetailsCode />
              </TabGroupItem>
              <TabGroupItem slug="qualifications">
                <template #label>
                  <Inline noWrap alignY="center">
                    <span>Qualifications</span>
                    <PillCounter
                      :count="selectedComponentFailingQualificationsCount"
                      tone="destructive"
                      hideIfZero
                      altStyle
                    />
                  </Inline>
                </template>
                <AssetQualificationsDetails
                  :componentId="selectedComponentId"
                />
              </TabGroupItem>

              <TabGroupItem label="Diff" slug="diff">
                <template #label>
                  <Inline noWrap alignY="center">
                    <span>Diff</span>
                    <StatusIndicatorIcon
                      v-if="selectedComponent.changeStatus !== 'unmodified'"
                      type="change"
                      :status="selectedComponent.changeStatus"
                    />
                  </Inline>
                </template>

                <AssetDiffDetails :componentId="selectedComponentId" />
              </TabGroupItem>
              <TabGroupItem label="Debug" slug="debug">
                <ComponentDebugDetails :componentId="selectedComponentId" />
              </TabGroupItem>
            </TabGroup>
          </TabGroupItem>
          <TabGroupItem slug="actions">
            <template #label>
              <Inline noWrap>
                <span>Actions</span>
                <PillCounter
                  :count="selectedComponentActionsCount"
                  hideIfZero
                />
              </Inline>
            </template>
            <AssetActionsDetails :componentId="selectedComponentId" />
          </TabGroupItem>
          <TabGroupItem slug="resource">
            <template #label>
              <Inline noWrap>
                <span>Resource</span>
                <StatusIndicatorIcon
                  v-if="selectedComponent.hasResource"
                  type="resource"
                  status="exists"
                />
              </Inline>
            </template>
            <ComponentDetailsResource />
          </TabGroupItem>
        </TabGroup>
      </div>
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref, watch } from "vue";
import {
  ErrorMessage,
  Icon,
  Inline,
  PillCounter,
  ScrollArea,
  Stack,
  TabGroup,
  TabGroupItem,
  VButton,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useStatusStore } from "@/store/status.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import { useActionsStore } from "@/store/actions.store";
import { ComponentType } from "@/api/sdf/dal/diagram";
import ComponentCard from "./ComponentCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import ComponentDetailsResource from "./ComponentDetailsResource.vue";
import ComponentDebugDetails from "./Debug/ComponentDebugDetails.vue";
import AssetQualificationsDetails from "./AssetQualificationsDetails.vue";
import AssetActionsDetails from "./AssetActionsDetails.vue";
import AssetDiffDetails from "./AssetDiffDetails.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import AttributesPanel from "./AttributesPanel/AttributesPanel.vue";
import ComponentDetailsCode from "./ComponentDetailsCode.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const emit = defineEmits<{
  (e: "delete"): void;
  (e: "restore"): void;
  (e: "openMenu", mouse: MouseEvent): void;
}>();

const componentsStore = useComponentsStore();
const qualificationsStore = useQualificationsStore();
const changeSetStore = useChangeSetsStore();
const actionsStore = useActionsStore();

const modelingEventBus = componentsStore.eventBus;

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
const selectedComponentFailingQualificationsCount = computed(
  () =>
    qualificationsStore.qualificationStatsByComponentId[
      selectedComponentId.value
    ]?.failed || 0,
);

const selectedComponentActionsCount = computed(() => {
  return _.filter(
    actionsStore.actionsByComponentId[selectedComponentId.value],
    (a) => !!a.actionInstanceId,
  ).length;
});

const statusStore = useStatusStore();
const currentStatus = computed(() =>
  selectedComponentId.value
    ? statusStore.getComponentStatus(selectedComponentId.value)
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

const tabsRef = ref<InstanceType<typeof TabGroup>>();
const componentSubTabsRef = ref<InstanceType<typeof TabGroup>>();

function onTabSelected(newTabSlug?: string) {
  componentsStore.setComponentDetailsTab(newTabSlug || null);
}

watch(
  () => componentsStore.selectedComponentDetailsTab,
  (tabSlug) => {
    if (tabSlug === "resource") {
      tabsRef.value?.selectTab("resource");
    } else if (tabSlug?.startsWith("actions")) {
      tabsRef.value?.selectTab("actions");
    } else {
      tabsRef.value?.selectTab("component");
      componentSubTabsRef.value?.selectTab(tabSlug || undefined);
    }
  },
);
</script>
