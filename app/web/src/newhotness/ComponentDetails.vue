<template>
  <ComponentDetailsSkeleton v-if="showSkeleton" />
  <section v-else :class="clsx('grid gap-sm h-full p-sm', gridStateClass)">
    <!-- Single banner area for all banner types -->
    <div
      v-if="
        showComponentStateBanner ||
        showStatusBanner ||
        (component && component.toDelete) ||
        (component && hasSocketConnection)
      "
      class="banner flex flex-col"
    >
      <!-- Status and deletion banners for existing components -->
      <template v-if="component">
        <!-- Status banner (template functions) -->
        <StatusBox
          v-if="showStatusBanner"
          :kind="
            specialCaseManagementExecutionStatus === 'Running' ||
            attributePanelRef?.importing
              ? 'loading'
              : 'error'
          "
          :text="statusBannerText"
        >
          <template #right>
            <NewButton
              v-if="specialCaseManagementFuncRun"
              :label="seeFuncRunLabel"
              @click="navigateToFuncRunDetails(specialCaseManagementFuncRun.id)"
            />
          </template>
        </StatusBox>

        <!-- Deletion banner (second highest priority) -->
        <div
          v-else-if="component.toDelete"
          :class="
            clsx(
              'flex flex-row items-center gap-xs px-sm py-xs',
              themeClasses('bg-neutral-300', 'bg-neutral-600'),
            )
          "
        >
          <NewButton
            v-tooltip="'Close (Esc)'"
            label="Marked for deletion"
            class="flex-none"
            @click="close"
          />
          <TruncateWithTooltip class="py-2xs text-sm">
            This component will be removed from HEAD once the current change set
            is applied.
          </TruncateWithTooltip>
        </div>

        <!-- Socket connections banner (lowest priority) -->
        <!-- TODO: Paul to get the light mode bg color -->
        <div
          v-else-if="hasSocketConnection"
          :class="
            clsx(
              'flex flex-row items-center gap-xs px-sm py-xs border',
              themeClasses(
                'bg-warning-50 border-warning-400 text-neutral-900',
                'border-warning-400 text-neutral-100',
              ),
            )
          "
          :style="{ backgroundColor: 'rgba(125, 74, 23, 0.25)' }"
        >
          <Icon
            name="alert-triangle-filled"
            size="sm"
            :class="themeClasses('text-warning-600', 'text-warning-300')"
          />
          <TruncateWithTooltip class="py-2xs text-sm flex-1">
            Some settings in this component are incompatible with the new
            experience. To learn how to update them, check out our
            documentation.
          </TruncateWithTooltip>
          <NewButton
            label="Learn more"
            tone="action"
            @click="openWorkspaceMigrationDocumentation"
          />
        </div>
      </template>
      <!-- No component banner -->
      <template v-else-if="showComponentExists">
        <div
          :class="
            clsx(
              'flex flex-row items-center gap-xs px-sm py-xs',
              themeClasses('bg-neutral-300', 'bg-neutral-600'),
            )
          "
        >
          <NewButton
            tooltip="Close (Esc)"
            tooltipPlacement="top"
            icon="x"
            tone="empty"
            :class="
              clsx(
                'active:bg-white active:text-black',
                themeClasses('hover:bg-neutral-400', 'hover:bg-neutral-500'),
              )
            "
            @click="close"
          />
          <TruncateWithTooltip v-if="!componentExists" class="py-xs text-sm">
            No component with id "{{ componentId }}" exists on this change set.
          </TruncateWithTooltip>
          <TruncateWithTooltip v-else class="py-xs text-sm">
            Component data is being retrieved...
            <Icon size="sm" name="loader" />
          </TruncateWithTooltip>
        </div>
      </template>
    </div>

    <template v-if="component">
      <div
        data-testid="component-name-section"
        :class="
          clsx(
            'name flex flex-row items-center gap-xs border-b',
            themeClasses(
              'bg-white border-neutral-300',
              'bg-neutral-800 border-neutral-600',
            ),
          )
        "
      >
        <NewButton
          tooltip="Close (Esc)"
          tooltipPlacement="top"
          icon="x"
          tone="empty"
          :class="
            clsx(
              'active:bg-white active:text-black',
              themeClasses('hover:bg-neutral-200', 'hover:bg-neutral-600'),
            )
          "
          @click="close"
        />
        <div class="flex-none text-sm">{{ component.schemaVariantName }}</div>
        <div class="flex-none">/</div>
        <TruncateWithTooltip
          class="flex-1 min-w-0 m-[-4px] py-2xs px-xs text-sm"
        >
          {{ component.name }}
        </TruncateWithTooltip>
        <div class="ml-auto flex flex-row gap-xs">
          <NewButton
            v-if="component.toDelete"
            v-tooltip="'Restore (R)'"
            label="Restore"
            tone="action"
            :loading="restoreLoading"
            loadingText="Restoring..."
            loadingIcon="loader"
            :disabled="restoreLoading"
            @click="restoreComponent"
          />
          <template v-else>
            <NewButton
              v-tooltip="'Erase (E)'"
              label="Erase"
              tone="destructive"
              @click="eraseComponent"
            />
            <NewButton
              v-tooltip="'Delete (⌫)'"
              label="Delete"
              tone="destructive"
              @click="deleteComponent"
            />
            <template v-if="isUpgradeable">
              <div class="w-px h-6 bg-neutral-600 self-center" />
              <NewButton
                v-tooltip="'Upgrade (U)'"
                :label="upgradeLoading ? 'Upgrading...' : 'Upgrade'"
                :icon="upgradeLoading ? 'loader' : 'bolt-outline'"
                :loading="upgradeLoading"
                :disabled="upgradeLoading"
                loadingIcon="loader"
                @click="upgradeComponent"
              />
            </template>
          </template>
        </div>
      </div>

      <div
        :class="
          clsx(
            'attrs flex flex-col',
            !showStatusBanner && !component?.toDelete && 'attrs-no-banner',
          )
        "
      >
        <CollapsingFlexItem ref="attrRef" :expandable="false" open>
          <template #header>
            <span class="text-sm">Attributes</span>
          </template>
          <template #headerIcons>
            <NewButton
              v-if="specialCaseManagementFuncKind === 'import'"
              v-tooltip="
                attributePanelRef?.importing
                  ? undefined
                  : 'Import Existing Resource (𝙸)'
              "
              label="Import a resource"
              :loading="attributePanelRef?.importing"
              loadingText="Importing..."
              @click.stop.prevent="openImportModal"
            />
            <NewButton
              v-else-if="
                specialCaseManagementFuncKind === 'runTemplate' &&
                specialCaseManagementFunc?.id
              "
              :label="
                specialCaseManagementExecutionStatus === 'Failure'
                  ? 'Re-run template'
                  : 'Run template'
              "
              tone="action"
              :loading="specialCaseManagementExecutionStatus === 'Running'"
              loadingText="Running template"
              :disabled="specialCaseManagementExecutionStatus === 'Running'"
              loadingIcon="loader"
              @click.stop="runMgmtFunc(specialCaseManagementFunc?.id)"
            />
          </template>
          <AttributePanel
            ref="attributePanelRef"
            :component="component"
            :attributeTree="attributeTree || undefined"
            :importFunc="importFunc"
            :importFuncRun="importFuncRun"
            @focused="(path) => focused(path)"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem
          v-if="hasResourceValueProps"
          ref="resourceRef"
          :expandable="false"
        >
          <template #header>
            <div class="flex place-content-between w-full">
              <span class="text-sm">Resource Values</span>
            </div>
          </template>
          <ResourceValuesPanel
            :component="component"
            :attributeTree="attributeTree || undefined"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem ref="actionRef" :expandable="false">
          <template #header><span class="text-sm">Actions</span></template>
          <ActionsPanel
            :component="component"
            :attributeValueId="component.rootAttributeValueId"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem ref="mgmtRef" :expandable="false">
          <template #header><span class="text-sm">Management</span></template>
          <ManagementPanel
            :component="component"
            :latestFuncRuns="latestFuncRuns"
          />
        </CollapsingFlexItem>
      </div>

      <div
        v-if="docsOpen"
        :class="
          clsx(
            'docs flex flex-col',
            !showStatusBanner && !component?.toDelete && 'docs-no-banner',
          )
        "
      >
        <DocumentationPanel
          :component="component"
          :docs="docs"
          :docLink="docLink"
          open
          @toggle="() => (docsOpen = false)"
          @cleardocs="() => (docs = '')"
        />
      </div>

      <div
        :class="
          clsx(
            'right flex flex-col',
            !showStatusBanner && !component?.toDelete && 'right-no-banner',
          )
        "
      >
        <CollapsingFlexItem>
          <template #header
            ><span class="text-sm">Qualifications</span></template
          >
          <template #headerIcons>
            <MinimizedComponentQualificationStatus
              :component="component"
              :dependentValues="dependentValues"
            />
          </template>
          <QualificationPanel
            :component="component"
            :attributeTree="attributeTree"
            :dependentValues="dependentValues"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem ref="connectionsFlex" expandable>
          <template #header
            ><span class="text-sm">Subscriptions</span></template
          >
          <template #headerIcons>
            <NewButton
              label="Visualize Subscriptions"
              truncateText
              @click="navigateToMap"
            />
          </template>
          <ConnectionsPanel
            v-if="componentConnections && component"
            ref="connectionsPanel"
            :component="component"
            :connections="componentConnections ?? undefined"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem expandable>
          <template #header><span class="text-sm">Code Gen</span></template>
          <CodePanel
            v-if="attributeTree"
            :component="component"
            :attributeTree="attributeTree"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem expandable>
          <template #header><span class="text-sm">Diff</span></template>
          <DiffPanel :component="component" />
        </CollapsingFlexItem>
        <CollapsingFlexItem expandable>
          <template #header><span class="text-sm">Resource</span></template>
          <template #headerIcons>
            <NewButton
              v-if="refreshEnabled"
              label="Refresh"
              :loading="refreshActionRunning"
              loadingIcon="loader"
              loadingText="Refreshing..."
              :disabled="refreshBifrosting"
              @click.stop="executeRefresh"
            />
            <Icon
              v-if="component.hasResource"
              name="check-hex"
              size="sm"
              tone="success"
            />
          </template>
          <ResourcePanel
            :component="component"
            :attributeTree="attributeTree ?? undefined"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem
          v-if="useFeatureFlagsStore().COMPONENT_HISTORY_FUNCS"
        >
          <template #header><span class="text-sm">History</span></template>
          <ComponentHistory
            :componentId="component.id"
            :enabled="componentExists"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem
          v-if="useFeatureFlagsStore().COMPONENT_HISTORY_FUNCS"
        >
          <template #header
            ><span class="text-sm">Function Runs</span></template
          >
          <ComponentFuncRunList
            :componentId="component.id"
            :enabled="componentExists"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem
          v-if="useFeatureFlagsStore().SQLITE_TOOLS"
          expandable
        >
          <template #header>
            <span class="text-sm">Debug</span>
          </template>
          <ComponentDebugPanel :componentId="component.id" />
        </CollapsingFlexItem>
        <DocumentationPanel
          v-if="!docsOpen"
          :component="component"
          :docs="docs"
          :docLink="docLink"
          @toggle="() => (docsOpen = true)"
        />
      </div>
    </template>
    <EraseModal ref="eraseModalRef" @confirm="componentsFinishErase" />
    <DeleteModal
      ref="deleteModalRef"
      @delete="(mode) => componentsFinishDelete(mode)"
    />
    <ComponentImportModal ref="importModalRef" @submit="doImport" />
  </section>
</template>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import {
  Icon,
  themeClasses,
  TruncateWithTooltip,
  NewButton,
} from "@si/vue-lib/design-system";
import {
  computed,
  ref,
  onMounted,
  onBeforeUnmount,
  inject,
  watch,
  useTemplateRef,
  nextTick,
} from "vue";
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import {
  bifrost,
  bifrostExists,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import {
  AttributeTree,
  BifrostComponent,
  DependentValues,
  EntityKind,
  IncomingConnections,
  MgmtFuncKind,
} from "@/workers/types/entity_kind_types";
import { ExploreContext } from "@/newhotness/types";
import { funcRunStatus, FuncRun } from "@/newhotness/api_composables/func_run";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ComponentDetailsSkeleton from "@/newhotness/skeletons/ComponentDetailsSkeleton.vue";
import AttributePanel from "./AttributePanel.vue";
import ResourceValuesPanel from "./ResourceValuesPanel.vue";
import {
  attributeEmitter,
  KeyDetails,
  keyEmitter,
} from "./logic_composables/emitters";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import StatusBox from "./layout_components/StatusBox.vue";
import { useApi, routes } from "./api_composables";
import QualificationPanel from "./QualificationPanel.vue";
import ResourcePanel from "./ResourcePanel.vue";
import CodePanel from "./CodePanel.vue";
import DiffPanel from "./DiffPanel.vue";
import ComponentHistory from "./ComponentHistory.vue";
import ComponentFuncRunList from "./ComponentFuncRunList.vue";
import ComponentDebugPanel from "./ComponentDebugPanel.vue";
import ActionsPanel from "./ActionsPanel.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";
import DocumentationPanel from "./DocumentationPanel.vue";
import ManagementPanel from "./ManagementPanel.vue";
import DeleteModal, { DeleteMode } from "./DeleteModal.vue";
import EraseModal from "./EraseModal.vue";
import MinimizedComponentQualificationStatus from "./MinimizedComponentQualificationStatus.vue";
import { useComponentDeletion } from "./composables/useComponentDeletion";
import { useComponentUpgrade } from "./composables/useComponentUpgrade";
import { useManagementFuncJobState } from "./logic_composables/management";
import { useComponentActions } from "./logic_composables/component_actions";
import { prevPage } from "./logic_composables/navigation_stack";
import { openWorkspaceMigrationDocumentation } from "./util";
import { useContext } from "./logic_composables/context";
import ComponentImportModal from "./ComponentImportModal.vue";

const showSkeleton = computed(
  () =>
    attributeTreeQuery.isLoading.value ||
    (!attributeTree.value && componentQuery.isLoading.value), // Prevent going back to skeleton after user changes
);

const props = defineProps<{
  componentId: string;
}>();

const realtimeStore = useRealtimeStore();

const ctx = useContext();
const explore = inject<ExploreContext | null>("EXPLORE_CONTEXT", null);

const key = useMakeKey();
const args = useMakeArgs();
const queryClient = useQueryClient();

const docsOpen = ref(true);

const componentId = computed(() => props.componentId);

const attributePanelRef = ref<InstanceType<typeof AttributePanel>>();

const componentQuery = useQuery<BifrostComponent | null>({
  queryKey: key(EntityKind.Component, componentId),
  queryFn: async (queryContext) => {
    const c = await bifrost<BifrostComponent>(
      args(EntityKind.Component, componentId.value),
    );
    if (c) return c;
    const d = queryContext.client.getQueryData(
      key(EntityKind.Component, componentId).value,
    ) as BifrostComponent | undefined;
    if (d) return d;
    return null;
  },
});

const enableExists = computed(
  () => componentQuery.isFetched && !componentQuery.data.value,
);
const componentExistsInIndexQuery = useQuery<boolean>({
  queryKey: key(EntityKind.Component, componentId, "exists"),
  queryFn: async () => {
    return await bifrostExists(args(EntityKind.Component, componentId.value));
  },
  enabled: enableExists,
});

const attributeTreeQuery = useQuery<AttributeTree | null>({
  queryKey: key(EntityKind.AttributeTree, componentId),
  queryFn: async (queryContext) => {
    const a = await bifrost<AttributeTree>(
      args(EntityKind.AttributeTree, componentId.value),
    );
    if (a) return a;
    const d = queryContext.client.getQueryData(
      key(EntityKind.AttributeTree, componentId).value,
    ) as AttributeTree | undefined;
    if (d) return d;
    return null;
  },
});
const attributeTree = computed(() => attributeTreeQuery.data.value);

const dependentValuesQuery = useQuery<DependentValues | null>({
  queryKey: key(EntityKind.DependentValues),
  queryFn: async () =>
    await bifrost<DependentValues>(args(EntityKind.DependentValues)),
});
const dependentValues = computed(() => dependentValuesQuery.data.value);

// Determines if the component is able to have a resource. For example, for "AWS::EC2::KeyPair",
// this would return 'true', but fore "AWS Region", this would return false.
const hasResourceValueProps = computed(() => {
  if (!attributeTree.value) return false;
  const propMatch = Object.entries(attributeTree.value.props).find(
    ([_, prop]) => prop.path === "root/resource_value",
  );
  if (!propMatch) return false;
  const attributeValueMatch = Object.entries(
    attributeTree.value.attributeValues,
  ).find(([_, attributeValue]) => attributeValue.propId === propMatch[0]);
  if (!attributeValueMatch) return false;
  const resourceValueSubtree =
    attributeTree.value.treeInfo[attributeValueMatch[0]];
  const subtreeChildCount = resourceValueSubtree?.children?.length;
  if (!subtreeChildCount) return false;
  return subtreeChildCount > 0;
});

const hasSocketConnection = computed(() => {
  if (!attributeTree.value) return false;
  return Object.values(attributeTree.value.attributeValues).some(
    (av) => av.hasSocketConnection,
  );
});

const component = computed(() => componentQuery.data.value || null);

const componentExists = computed(
  () => !!component.value || (componentExistsInIndexQuery.data.value ?? false),
);

const showComponentExists = computed(
  () => !component.value && componentExistsInIndexQuery.isFetched.value,
);

// Actions composable - reactive to component changes
const { refreshEnabled, refreshActionRunning, runRefreshHandler } =
  useComponentActions(component);
const { executeRefresh, bifrosting: refreshBifrosting } = runRefreshHandler();
const mgmtFuncs = computed(
  () => component.value?.schemaVariant.mgmtFunctions ?? [],
);

const componentConnectionsQuery = useQuery<IncomingConnections | null>({
  queryKey: key(EntityKind.IncomingConnections, componentId),
  queryFn: async () => {
    const incomingConnections = await bifrost<IncomingConnections>(
      args(EntityKind.IncomingConnections, componentId.value),
    );
    return incomingConnections;
  },
});
const componentConnections = computed(
  () => componentConnectionsQuery.data.value,
);

const connectionsPanel =
  useTemplateRef<typeof ConnectionsPanel>("connectionsPanel");
const connectionsFlex =
  useTemplateRef<typeof CollapsingFlexItem>("connectionsFlex");
const focused = async (path: string) => {
  if (connectionsFlex.value) connectionsFlex.value.openState.open.value = true;
  await nextTick();
  connectionsPanel.value?.highlight(path);
};

const docs = ref("");
const docLink = ref("");

attributeEmitter.on("selectedDocs", (data) => {
  if (!data) docs.value = "";
  else {
    docs.value = data.docs;
    docLink.value = data.link;
  }
});

const attrRef = ref<typeof CollapsingFlexItem>();
const resourceRef = ref<typeof CollapsingFlexItem>();
const actionRef = ref<typeof CollapsingFlexItem>();
const mgmtRef = ref<typeof CollapsingFlexItem>();

const router = useRouter();

const close = () => {
  const lastPage = prevPage();

  // If we have a previous page and it's the explore view, go back to it with full context
  if (lastPage && lastPage.name === "new-hotness") {
    router.push({
      name: lastPage.name,
      params: lastPage.params,
      query: lastPage.query,
    });
  } else {
    // Fallback to default navigation if no history
    const params = router.currentRoute?.value.params ?? {};
    delete params.componentId;
    router.push({
      name: "new-hotness",
      params,
      query: { retainSessionState: 1 },
    });
  }
};

const navigateToMap = () => {
  const params = router.currentRoute?.value.params ?? {};
  const query = { ...router.currentRoute?.value.query };

  delete params.componentId;
  delete query.grid;
  delete query.retainSessionState;
  query.map = "1";
  if (component.value?.id) {
    query.c = component.value.id;
    // Add flag to hide unconnected components
    // This is a specific path through to the Map to show only the items
    // that would effectively show in a pin
    query.hideSubscriptions = "1";
  }

  router.push({
    name: "new-hotness",
    params,
    query,
  });
};

const api = useApi();

export type NameFormData = {
  name: string;
};

// Special case management funcs
const specialCaseManagementFunc = computed(() => {
  const importFunc = mgmtFuncs.value.find(
    (f) => f.kind === MgmtFuncKind.Import,
  );
  const runTemplateFunc = mgmtFuncs.value.find(
    (f) => f.kind === MgmtFuncKind.RunTemplate,
  );
  if (importFunc) return importFunc; // chose import first if both appear (they shouldn't!)
  if (runTemplateFunc) return runTemplateFunc;
  return undefined;
});
const specialCaseManagementFuncKind = computed(() => {
  if (specialCaseManagementFunc.value?.kind === MgmtFuncKind.Import)
    return "import";
  if (specialCaseManagementFunc.value?.kind === MgmtFuncKind.RunTemplate)
    return "runTemplate";
  return undefined;
});
const dispatchedSpecialCaseManagementFunc = ref(false);

const importFunc = computed(() =>
  specialCaseManagementFuncKind.value === "import"
    ? specialCaseManagementFunc.value
    : undefined,
);
const importFuncRun = computed(() =>
  specialCaseManagementFuncKind.value === "import"
    ? specialCaseManagementFuncRun.value
    : undefined,
);

// API to run special case management funcs
const mgmtRunApi = useApi();
const route = useRoute();
const runMgmtFunc = async (funcId: string) => {
  const call = mgmtRunApi.endpoint<{ success: boolean }>(routes.MgmtFuncRun, {
    prototypeId: funcId,
    componentId: props.componentId,
    viewId: "DEFAULT",
  });

  const { req, newChangeSetId } = await call.post({});

  dispatchedSpecialCaseManagementFunc.value = true;
  setTimeout(() => {
    dispatchedSpecialCaseManagementFunc.value = false;
  }, 2000);

  // NOTE(nick): need to make sure this makes sense after the timeout.
  if (mgmtRunApi.ok(req) && newChangeSetId) {
    mgmtRunApi.navigateToNewChangeSet(
      {
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.componentId,
        },
      },
      newChangeSetId,
    );
  }
};

const importModalRef = ref<InstanceType<typeof ComponentImportModal>>();
const openImportModal = () => {
  if (attributePanelRef.value?.resourceIdValue) {
    importModalRef.value?.open(
      attributePanelRef.value?.resourceIdValue as string,
    );
  } else {
    importModalRef.value?.open();
  }
};

// MGMT funcs
const MGMT_RUN_KEY = "latestMgmtFuncRuns";

const funcRunQuery = useQuery({
  enabled: () => componentExists.value,
  queryKey: [ctx.changeSetId, MGMT_RUN_KEY],
  queryFn: async () =>
    api
      .endpoint<FuncRun[]>(routes.MgmtFuncGetLatest, {
        componentId: componentId.value,
      })
      .get(),
});

const funcRuns = computed<FuncRun[]>(() => {
  if (!funcRunQuery.data.value) return [];
  return funcRunQuery.data.value.data;
});

// The latest funcrun for this each mgmt prototype of this component, keyed by the prototypeId
const latestFuncRuns = computed(() => {
  const runs = {} as Record<string, FuncRun>;

  if (!componentId.value || !componentExists.value) return runs;

  for (const funcRun of funcRuns.value) {
    if (funcRun.functionKind !== "Management") continue;
    if (funcRun.componentId !== componentId.value) continue;
    if (!funcRun.actionPrototypeId) continue;

    const maybeRun = runs[funcRun.actionPrototypeId];

    if (!maybeRun) {
      runs[funcRun.actionPrototypeId] = funcRun;
    } else {
      if (maybeRun.createdAt < funcRun.createdAt) {
        runs[funcRun.actionPrototypeId] = funcRun;
      }
    }
  }

  return runs;
});

// If any mgmt func for this component is running, query the status every 5 seconds
// Ideally the websocket requests will give us faster updates, but this is a failsafe
watch([funcRuns], () => {
  if (funcRuns.value.find((run) => run.state === "Running")) {
    setTimeout(() => {
      queryClient.invalidateQueries({
        queryKey: [ctx.changeSetId, MGMT_RUN_KEY],
      });
    }, 5000);
  }
});

// Special case func run and state
const specialCaseManagementFuncRun = computed(() => {
  const specialCaseManagementFuncId = specialCaseManagementFunc.value?.id;
  if (!specialCaseManagementFuncId) return undefined;
  return latestFuncRuns.value[specialCaseManagementFuncId];
});
const specialCaseManagementFuncJobStateComposable = useManagementFuncJobState(
  specialCaseManagementFuncRun,
);
const specialCaseManagementFuncJobState = computed(
  () => specialCaseManagementFuncJobStateComposable.value.value,
);
const specialCaseManagementExecutionStatus = computed(() => {
  if (dispatchedSpecialCaseManagementFunc.value) return "Running";
  return funcRunStatus(
    specialCaseManagementFuncRun.value,
    specialCaseManagementFuncJobState.value?.state,
  );
});

// When absolutely anything in the component changes, invalidate the audit logs query for that component.
watch(
  component,
  (newComponent) => {
    if (newComponent) {
      queryClient.invalidateQueries({
        queryKey: key(EntityKind.AuditLogsForComponent, newComponent.id).value,
      });
    }
  },
  { deep: true },
);

const onBackspace = () => {
  if (!component.value?.toDelete) {
    deleteComponent();
  }
};

const shortcuts: { [Key in string]: (e: KeyDetails[Key]) => void } = {
  // a: used for select all in Explore, not used here
  // b: undefined,
  c: (e) => {
    if (e.metaKey || e.ctrlKey) return; // copy
    e.preventDefault();
    emit("openChangesetModal");
  },
  // d: used for duplicate in Explore, not used here
  e: (e) => {
    e.preventDefault();
    if (!component.value?.toDelete) {
      eraseComponent();
    }
  },
  f: (e) => {
    if (e.metaKey || e.ctrlKey) return; // find
    e.preventDefault();
    if (component.value?.toDelete) {
      restoreComponent();
    }
  },
  // g: undefined,
  // h: undefined,
  i: (e) => {
    if (e.metaKey || e.ctrlKey) return;
    e.preventDefault();
    if (specialCaseManagementFuncKind.value === "import") {
      openImportModal();
    }
  },
  // j: undefined,
  k: (e) => {
    e.preventDefault();
    // k focuses the search on Explore, so why not have it focus the search here too?
    attributePanelRef.value?.focusSearch();
  },
  // l: undefined,
  // m: used to toggle the minimap in the Map view
  // n: used to open the add component modal in Explore
  // o: undefined,
  // p: used to pin a component in the Grid view
  // q: undefined,
  r: (e) => {
    if (e.metaKey || e.ctrlKey) {
      // This is the chrome hotkey combo for refreshing the page! Let it happen!
      return;
    } else if (ctx.onHead.value) {
      // Can't open the review screen on Head
      return;
    }

    e.preventDefault();
    router.push({
      name: "new-hotness-review",
    });
  },
  // s: undefined,
  // t: used on the Grid view to open the create template modal
  u: () => {
    if (!component.value?.toDelete && isUpgradeable.value) {
      upgradeComponent();
    }
  },
  // w: undefined,
  // x: undefined,
  // y: undefined,
  // z: undefined,
  Backspace: onBackspace,
  Delete: onBackspace,
  Escape: () => {
    close();
  },
};

onMounted(() => {
  for (const [key, func] of Object.entries(shortcuts)) {
    keyEmitter.on(key, func);
  }
  realtimeStore.subscribe(MGMT_RUN_KEY, `changeset/${ctx.changeSetId.value}`, [
    {
      eventType: "FuncRunLogUpdated",
      callback: async (payload) => {
        if (mgmtFuncs.value.find((m) => m.funcId === payload.actionId)) {
          setTimeout(() => {
            queryClient.invalidateQueries({
              queryKey: [ctx.changeSetId, MGMT_RUN_KEY],
            });
          }, 500);
        }
      },
    },
  ]);
});
onBeforeUnmount(() => {
  for (const [key, func] of Object.entries(shortcuts)) {
    keyEmitter.off(key, func);
  }
  realtimeStore.unsubscribe(MGMT_RUN_KEY);
});

const importFailure = computed(
  () =>
    specialCaseManagementFuncKind.value === "import" &&
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (specialCaseManagementFuncRun.value?.resultValue as any)?.health ===
      "error",
);
const showComponentStateBanner = computed(() => !component.value);
const showStatusBanner = computed(
  () =>
    importFailure.value ||
    specialCaseManagementExecutionStatus.value === "Failure" ||
    specialCaseManagementExecutionStatus.value === "ActionFailure" ||
    specialCaseManagementExecutionStatus.value === "Running",
);
const seeFuncRunLabel = "See Func Run";
const statusBannerText = computed(() => {
  if (specialCaseManagementFuncKind.value === "import")
    if (
      specialCaseManagementExecutionStatus.value === "Running" ||
      attributePanelRef.value?.importing
    )
      return "Importing...";
    else
      return `Error executing Import function. Click "${seeFuncRunLabel}" for more details.`;
  if (specialCaseManagementFuncKind.value === "runTemplate")
    if (specialCaseManagementExecutionStatus.value === "Running")
      return "Extracting components from the template...";
    else
      return `Error executing Run Template function. Click "${seeFuncRunLabel}" for more details.`;
  return "";
});

const gridStateClass = computed(() => {
  // When there's no component, we only show the banner
  if (showComponentStateBanner.value) {
    return "no-component";
  }

  const hasBanner =
    showStatusBanner.value ||
    component.value?.toDelete ||
    hasSocketConnection.value;

  if (docsOpen.value) {
    return hasBanner ? "docs-open-with-banner" : "docs-open-without-banner";
  } else {
    return hasBanner ? "docs-closed-with-banner" : "docs-closed-without-banner";
  }
});

const deleteModalRef = ref<InstanceType<typeof DeleteModal>>();
const eraseModalRef = ref<InstanceType<typeof EraseModal>>();
const {
  convertBifrostToComponentInList,
  deleteComponents,
  eraseComponents,
  restoreComponents,
} = useComponentDeletion(undefined, true);

const { upgradeComponents } = useComponentUpgrade();

const isUpgradeable = computed(() => {
  if (!component.value) return false;

  if (component.value.canBeUpgraded !== undefined) {
    return component.value.canBeUpgraded;
  }

  // Fallback to explore context for upgrade info
  if (explore) {
    return explore.upgradeableComponents.value.has(component.value.id);
  }

  return false;
});

const deleteComponent = () => {
  if (!component.value) return;
  const componentForModal = convertBifrostToComponentInList(component.value);
  deleteModalRef.value?.open([componentForModal]);
};

const eraseComponent = () => {
  if (!component.value) return;
  const componentForModal = convertBifrostToComponentInList(component.value);
  eraseModalRef.value?.open([componentForModal]);
};

const upgradeLoading = ref(false);
const restoreLoading = ref(false);

const upgradeComponent = async () => {
  if (!component.value || upgradeLoading.value) return;

  upgradeLoading.value = true;
  await upgradeComponents([component.value.id]);
  upgradeLoading.value = false;
};

const restoreComponent = async () => {
  if (!component.value || restoreLoading.value) return;

  restoreLoading.value = true;
  const result = await restoreComponents([component.value.id]);
  if (result.success) {
    if (result.newChangeSetId) {
      // Navigate to the same component details page in the new change set
      router.push({
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: result.newChangeSetId,
          componentId: component.value.id,
        },
      });
    } else {
      close();
    }
  }
  restoreLoading.value = false;
};

const componentsFinishErase = async () => {
  if (!component.value) return;
  const result = await eraseComponents([component.value.id]);
  if (result.success) {
    eraseModalRef.value?.close();
    close();
  }
};

const componentsFinishDelete = async (mode: DeleteMode) => {
  if (!component.value) return;
  const result = await deleteComponents([component.value.id], mode);
  if (result.success) {
    deleteModalRef.value?.close();
    close();
  }
};

const navigateToFuncRunDetails = (funcRunId: string) => {
  router.push({
    name: "new-hotness-func-run",
    params: {
      workspacePk: route.params.workspacePk,
      changeSetId: route.params.changeSetId,
      funcRunId,
    },
  });
};

const doImport = (resourceId: string) => {
  attributePanelRef.value?.doImport(resourceId);
};

const emit = defineEmits<{
  (e: "openChangesetModal"): void;
}>();
</script>

<style lang="less" scoped>
section.grid.docs-open-with-banner {
  grid-template-areas:
    "name name name"
    "banner banner banner"
    "attrs docs right";
  grid-template-rows: 2.5rem auto minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
}
section.grid.docs-closed-with-banner {
  grid-template-areas:
    "name name"
    "banner banner"
    "attrs right";
  grid-template-rows: 2.5rem auto minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 33%);
}
section.grid.docs-open-without-banner {
  grid-template-areas:
    "name name name"
    "attrs docs right";
  grid-template-rows: 2.5rem minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
}
section.grid.docs-closed-without-banner {
  grid-template-areas:
    "name name"
    "attrs right";
  grid-template-rows: 2.5rem minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 33%);
}
section.grid.no-component {
  grid-template-areas: "banner";
  grid-template-rows: auto;
  grid-template-columns: 1fr;
}
.docs {
  grid-area: docs;
}
.right {
  grid-area: right;
}
.name {
  grid-area: name;
  margin: -0.75rem -1rem 0 -1rem;
  margin-top: -1em;
  padding: 0 0.5rem 0 0.5rem;
  height: 2.75rem;
}
.attrs {
  grid-area: attrs;
}
.attrs-no-banner {
  margin-top: -0.75rem;
}
.docs-no-banner {
  margin-top: -0.75rem;
}
.right-no-banner {
  margin-top: -0.75rem;
}
.banner {
  grid-area: banner;
  margin-top: -0.75rem;
}
</style>
