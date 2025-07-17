<template>
  <ScrollArea>
    <template #top>
      <ComponentCard :component="props.component" titleCard class="mb-xs">
        <DetailsPanelMenuIcon
          :selected="props.menuSelected"
          @click="
            (e) => {
              emit('openMenu', e);
            }
          "
        />
      </ComponentCard>
      <div
        v-if="isUpdating"
        class="flex flex-row items-center gap-xs m-xs mt-0"
      >
        <!-- currently updating -->
        <Icon name="loader" size="xs" class="text-action-500 shrink-0" />
        <div class="grow truncate text-xs italic">Updating...</div>
      </div>
      <div
        v-else
        :class="
          clsx('flex flex-row items-center', showRefreshButton && 'ml-xs mb-xs')
        "
      >
        <DetailsPanelTimestamps
          :changeStatus="props.component.def.changeStatus"
          :created="props.component.def.createdInfo"
          :modified="props.component.def.updatedInfo"
          :deleted="props.component.def.deletedInfo"
          :noMargin="showRefreshButton"
        />
        <div class="pr-xs shrink-0">
          <VButton
            v-if="showRefreshButton"
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

    <template v-if="props.component.def.changeStatus === 'deleted'">
      <Stack v-if="!changeSetsStore.headSelected" class="p-sm">
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
            props.component.def.componentType === ComponentType.Component
              ? 'Component'
              : 'Frame'
          }`"
          @click="modelingEventBus.emit('restoreSelection')"
        />
      </Stack>
      <Stack v-else class="p-sm">
        <ErrorMessage icon="alert-triangle" tone="warning">
          This component will be removed from your model once the delete action
          has completed.
        </ErrorMessage>
      </Stack>
    </template>

    <template v-else>
      <div class="absolute inset-0 border-t dark:border-neutral-700">
        <TabGroup
          ref="tabsRef"
          trackingSlug="asset_details"
          :startSelectedTabSlug="viewStore.detailsTabSlugs[0] || undefined"
          @update:selectedTab="onTabSelected"
        >
          <TabGroupItem slug="component">
            <template #label>
              <Inline noWrap alignY="center">
                <span class="uppercase">Component</span>
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
              variant="secondary"
              :startSelectedTabSlug="viewStore.detailsTabSlugs[1] || undefined"
              marginTop="2xs"
              @update:selectedTab="onTabSelected"
            >
              <TabGroupItem label="Attributes" slug="attributes">
                <AttributesPanel />
              </TabGroupItem>
              <TabGroupItem label="Connections" slug="connections">
                <ComponentConnectionsPanel />
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
                      toneToBg
                      class="text-shade-0"
                    />
                  </Inline>
                </template>
                <AssetQualificationsDetails :component="props.component" />
              </TabGroupItem>

              <TabGroupItem label="Diff" slug="diff">
                <template #label>
                  <Inline noWrap alignY="center">
                    <span>Diff</span>
                    <StatusIndicatorIcon
                      v-if="props.component.def.changeStatus !== 'unmodified'"
                      type="change"
                      :status="props.component.def.changeStatus"
                    />
                  </Inline>
                </template>

                <AssetDiffDetails :component="props.component" />
              </TabGroupItem>
              <TabGroupItem label="Debug" slug="debug">
                <ComponentDebugDetails :component="props.component" />
              </TabGroupItem>
            </TabGroup>
          </TabGroupItem>
          <TabGroupItem slug="resource">
            <template #label>
              <Inline noWrap alignY="center">
                <span class="uppercase">Resource</span>
                <StatusIndicatorIcon
                  v-if="props.component.def.hasResource"
                  type="resource"
                  status="exists"
                  size="sm"
                />
              </Inline>
            </template>
            <template
              v-if="
                featureFlagsStore.FRONTEND_ARCH_VIEWS &&
                featureFlagsStore.BIFROST_ACTIONS &&
                viewStore.selectedComponentId
              "
            >
              <BifrostAssetActionsDetails :component="component" />
            </template>
            <template v-else>
              <AssetActionsDetails :component="props.component" />
            </template>
          </TabGroupItem>
          <TabGroupItem
            v-if="funcStore.managementFunctionsForSelectedComponent.length > 0"
            slug="management"
            label="Mgmt Fns"
          >
            <ComponentDetailsManagement :component="props.component" />
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
import clsx from "clsx";
import { useComponentsStore } from "@/store/components.store";
import { useStatusStore } from "@/store/status.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useQualificationsStore } from "@/store/qualifications.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useFuncStore } from "@/store/func/funcs.store";
import { useViewsStore } from "@/store/views.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import BifrostAssetActionsDetails from "@/mead-hall/AssetActionsDetails.vue";
import ComponentCard from "./ComponentCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import ComponentDetailsManagement from "./ComponentDetailsManagement.vue";
import ComponentDebugDetails from "./Debug/ComponentDebugDetails.vue";
import AssetQualificationsDetails from "./AssetQualificationsDetails.vue";
import AssetActionsDetails from "./AssetActionsDetails.vue";
import AssetDiffDetails from "./AssetDiffDetails.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import AttributesPanel from "./AttributesPanel/AttributesPanel.vue";
import ComponentDetailsCode from "./ComponentDetailsCode.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
} from "./ModelingDiagram/diagram_types";
import ComponentConnectionsPanel from "./ComponentConnectionsPanel.vue";

const props = defineProps<{
  menuSelected: boolean;
  component: DiagramNodeData | DiagramGroupData;
}>();

const emit = defineEmits<{
  (e: "delete"): void;
  (e: "restore"): void;
  (e: "openMenu", mouse: MouseEvent): void;
}>();

const componentsStore = useComponentsStore();
const viewStore = useViewsStore();
const qualificationsStore = useQualificationsStore();
const changeSetsStore = useChangeSetsStore();
const funcStore = useFuncStore();
const featureFlagsStore = useFeatureFlagsStore();

const modelingEventBus = componentsStore.eventBus;

const selectedComponentQualificationStatus = computed(
  () =>
    qualificationsStore.qualificationStatusByComponentId[
      props.component.def.id
    ],
);
const selectedComponentFailingQualificationsCount = computed(
  () =>
    qualificationsStore.qualificationStatsByComponentId[props.component.def.id]
      ?.failed || 0,
);

const statusStore = useStatusStore();
const isUpdating = computed(
  () => statusStore.activeComponents[props.component.def.id],
);

const refreshing = computed(() => {
  return componentsStore.refreshingStatus[props.component.def.id] ?? false;
});

const onClickRefreshButton = () => {
  componentsStore.REFRESH_RESOURCE_INFO(props.component.def.id);
};

const tabsRef = ref<InstanceType<typeof TabGroup>>();
const componentSubTabsRef = ref<InstanceType<typeof TabGroup>>();

function onTabSelected(newTabSlug?: string) {
  viewStore.setComponentDetailsTab(newTabSlug || null);
}

const showRefreshButton = computed(
  () =>
    props.component.def.hasResource &&
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId,
);

watch(
  () => viewStore.selectedComponentDetailsTab,
  (tabSlug) => {
    if (tabSlug?.startsWith("resource")) {
      tabsRef.value?.selectTab("resource");
    } else if (tabSlug?.startsWith("management")) {
      tabsRef.value?.selectTab("management");
    } else {
      tabsRef.value?.selectTab("component");
      componentSubTabsRef.value?.selectTab(tabSlug || "attributes");
    }
  },
);
</script>
