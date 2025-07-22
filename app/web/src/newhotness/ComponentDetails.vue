<template>
  <DelayedLoader v-if="componentQuery.isLoading.value" :size="'full'" />
  <section v-else :class="clsx('grid gap-sm h-full p-sm', gridStateClass)">
    <div
      v-if="showBanner"
      :class="
        clsx(
          'banner',
          'flex flex-row items-center gap-xs px-xs ',
          themeClasses('bg-neutral-300', 'bg-neutral-600'),
        )
      "
    >
      <template v-if="!component">
        <IconButton
          tooltip="Close (Esc)"
          tooltipPlacement="top"
          class="border-0 mr-2em"
          icon="x"
          size="sm"
          iconIdleTone="shade"
          iconTone="shade"
          @click="close"
        />
        <TruncateWithTooltip class="py-xs">
          No component with id "{{ componentId }}" exists on this change set.
        </TruncateWithTooltip>
      </template>
      <template v-else-if="component.toDelete">
        <VButton
          v-tooltip="'Close (Esc)'"
          label="Marked for deletion"
          size="sm"
          tone="neutral"
          class="font-normal !py-0 flex-none"
          @click="close"
        />
        <TruncateWithTooltip class="py-xs">
          This component will be removed from HEAD once the current change set
          is applied.
        </TruncateWithTooltip>
        <!-- TODO(Wendy) - make this button work -->
        <!-- <VButton
          label="Restore Component"
          size="sm"
          tone="neutral"
          :class="
            clsx(
              'font-normal !py-0 flex-none ml-auto',
              themeClasses(
                '!bg-neutral-500 hover:!bg-neutral-400',
                '!bg-neutral-700 hover:!bg-neutral-800',
              ),
            )
          "
        /> -->
      </template>
    </div>

    <template v-if="component">
      <div
        :class="
          clsx(
            'name flex flex-row items-center gap-xs p-xs border',
            themeClasses(
              'bg-white border-neutral-300',
              'bg-neutral-800 border-neutral-600',
            ),
          )
        "
      >
        <IconButton
          tooltip="Close (Esc)"
          tooltipPlacement="top"
          class="border-0 mr-2em"
          icon="x"
          size="sm"
          iconIdleTone="shade"
          iconTone="shade"
          @click="close"
        />
        <div class="flex-none">{{ component.schemaVariantName }}</div>
        <div class="flex-none">/</div>
        <TruncateWithTooltip class="flex-1 min-w-0 m-[-4px] py-2xs px-xs">{{
          component.name
        }}</TruncateWithTooltip>
        <div class="ml-auto flex gap-xs">
          <template v-if="component.toDelete">
            <VButton
              v-tooltip="'Restore (R)'"
              size="sm"
              tone="success"
              :label="restoreLoading ? 'Restoring...' : 'Restore'"
              variant="ghost"
              :icon="restoreLoading ? 'loader' : 'trash-restore'"
              :loading="restoreLoading"
              :disabled="restoreLoading"
              loadingIcon="loader"
              @click="restoreComponent"
            />
          </template>
          <template v-else>
            <VButton
              v-tooltip="'Erase (E)'"
              size="sm"
              tone="destructive"
              label="Erase"
              variant="ghost"
              @click="eraseComponent"
            />
            <VButton
              v-tooltip="'Delete (⌫)'"
              size="sm"
              tone="destructive"
              label="Delete"
              class="border border-destructive-500 bg-destructive-500 text-white hover:bg-destructive-600 hover:border-destructive-600"
              @click="deleteComponent"
            />
            <VButton
              v-if="isUpgradeable"
              v-tooltip="'Upgrade (U)'"
              size="sm"
              tone="action"
              :label="upgradeLoading ? 'Upgrading...' : 'Upgrade'"
              variant="ghost"
              :icon="upgradeLoading ? 'loader' : 'bolt-outline'"
              :loading="upgradeLoading"
              :disabled="upgradeLoading"
              loadingIcon="loader"
              @click="upgradeComponent"
            />
            <VButton
              v-if="runTemplateFunc"
              size="sm"
              tone="neutral"
              :label="
                dispatchedFunc ||
                latestFuncRuns[runTemplateFunc?.id]?.state === 'Running'
                  ? 'Running...'
                  : 'Run Template'
              "
              variant="ghost"
              :loading="
                dispatchedFunc ||
                latestFuncRuns[runTemplateFunc?.id]?.state === 'Running'
              "
              :disabled="
                dispatchedFunc ||
                latestFuncRuns[runTemplateFunc?.id]?.state === 'Running'
              "
              loadingIcon="loader"
              @click="runMgmtFunc(runTemplateFunc?.id)"
            />
          </template>
        </div>
      </div>

      <div class="attrs flex flex-col gap-sm">
        <CollapsingFlexItem ref="attrRef" :expandable="false" open>
          <template #header>
            <div class="flex place-content-between w-full">
              <span>Attributes</span>
              <template v-if="importFunc">
                <VButton
                  size="xs"
                  tone="neutral"
                  :label="
                    showResourceInput
                      ? 'Set attributes manually'
                      : 'Import a Resource'
                  "
                  class="font-normal p-0 h-md mt-[1px] [&>div]:top-[-2px]"
                  @click.stop="
                    () => {
                      showResourceInput = !showResourceInput;
                    }
                  "
                />
              </template>
            </div>
          </template>
          <AttributePanel
            :component="component"
            :attributeTree="attributeTree"
            :importFunc="showResourceInput ? importFunc : undefined"
            :importFuncRun="latestFuncRuns[importFunc?.id ?? '']"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem
          v-if="hasResourceValueProps"
          ref="resourceRef"
          :expandable="false"
        >
          <template #header>
            <div class="flex place-content-between w-full">
              <span>Resource Values</span>
            </div>
          </template>
          <ResourceValuesPanel
            :component="component"
            :attributeTree="attributeTree"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem ref="actionRef" :expandable="false">
          <template #header>Actions</template>
          <ActionsPanel
            :component="component"
            :attributeValueId="component.rootAttributeValueId"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem ref="mgmtRef" :expandable="false">
          <template #header>Management</template>
          <ManagementPanel
            :component="component"
            :latestFuncRuns="latestFuncRuns"
          />
        </CollapsingFlexItem>
      </div>

      <div v-if="docsOpen" class="docs flex flex-col">
        <DocumentationPanel
          :component="component"
          :docs="docs"
          :docLink="docLink"
          open
          @toggle="() => (docsOpen = false)"
          @cleardocs="() => (docs = '')"
        />
      </div>

      <div class="right flex flex-col">
        <CollapsingFlexItem>
          <template #header> Qualifications </template>
          <template #headerIcons>
            <MinimizedComponentQualificationStatus :component="component" />
          </template>
          <QualificationPanel
            :component="component"
            :attributeTree="attributeTree"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem expandable>
          <template #header> Connections </template>
          <template #headerIcons></template>
          <ConnectionsPanel
            v-if="componentConnections && component"
            :component="component"
            :connections="componentConnections ?? undefined"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem expandable>
          <template #header> Code Gen </template>
          <CodePanel
            v-if="attributeTree"
            :component="component"
            :attributeTree="attributeTree"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem expandable>
          <template #header> Diff </template>
          <DiffPanel :component="component" />
        </CollapsingFlexItem>
        <CollapsingFlexItem expandable>
          <template #header> Resource </template>
          <template #headerIcons>
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
  </section>
</template>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import {
  VButton,
  Icon,
  themeClasses,
  IconButton,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { computed, ref, onMounted, onBeforeUnmount, inject, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  AttributeTree,
  BifrostComponent,
  EntityKind,
  IncomingConnections,
  MgmtFuncKind,
} from "@/workers/types/entity_kind_types";
import { Context, ExploreContext, assertIsDefined } from "@/newhotness/types";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import AttributePanel from "./AttributePanel.vue";
import ResourceValuesPanel from "./ResourceValuesPanel.vue";
import { attributeEmitter, keyEmitter } from "./logic_composables/emitters";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import DelayedLoader from "./layout_components/DelayedLoader.vue";
import { useApi, routes } from "./api_composables";
import QualificationPanel from "./QualificationPanel.vue";
import ResourcePanel from "./ResourcePanel.vue";
import CodePanel from "./CodePanel.vue";
import DiffPanel from "./DiffPanel.vue";
import ActionsPanel from "./ActionsPanel.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";
import DocumentationPanel from "./DocumentationPanel.vue";
import ManagementPanel from "./ManagementPanel.vue";
import DeleteModal, { DeleteMode } from "./DeleteModal.vue";
import EraseModal from "./EraseModal.vue";
import MinimizedComponentQualificationStatus from "./MinimizedComponentQualificationStatus.vue";
import { useComponentDeletion } from "./composables/useComponentDeletion";
import { useComponentUpgrade } from "./composables/useComponentUpgrade";

const props = defineProps<{
  componentId: string;
}>();

const realtimeStore = useRealtimeStore();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);
const explore = inject<ExploreContext | null>("EXPLORE_CONTEXT", null);

const key = useMakeKey();
const args = useMakeArgs();
const queryClient = useQueryClient();

const docsOpen = ref(true);

const componentId = computed(() => props.componentId);

const componentQuery = useQuery<BifrostComponent | undefined>({
  queryKey: key(EntityKind.Component, componentId),
  queryFn: async (queryContext) =>
    (await bifrost<BifrostComponent>(
      args(EntityKind.Component, componentId.value),
    )) ??
    queryContext.client.getQueryData(
      key(EntityKind.Component, componentId).value,
    ),
});

const attributeTreeQuery = useQuery<AttributeTree | undefined>({
  queryKey: key(EntityKind.AttributeTree, componentId.value),
  queryFn: async (queryContext) =>
    (await bifrost<AttributeTree>(
      args(EntityKind.AttributeTree, componentId.value),
    )) ??
    queryContext.client.getQueryData(
      key(EntityKind.AttributeTree, componentId).value,
    ),
});
const attributeTree = computed(() => attributeTreeQuery.data.value);

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

const component = computed(() => componentQuery.data.value);

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
  const params = router.currentRoute?.value.params ?? {};
  delete params.componentId;
  router.push({
    name: "new-hotness",
    params,
    query: { retainSessionState: 1 },
  });
};

const api = useApi();

export type NameFormData = {
  name: string;
};

// Import
const importFunc = computed(() =>
  mgmtFuncs.value.find((f) => f.kind === MgmtFuncKind.Import),
);

// Run Template
const runTemplateFunc = computed(() =>
  mgmtFuncs.value.find((f) => f.kind === MgmtFuncKind.RunTemplate),
);
const dispatchedFunc = ref(false);

const mgmtRunApi = useApi();
const route = useRoute();
const runMgmtFunc = async (funcId: string) => {
  const call = mgmtRunApi.endpoint<{ success: boolean }>(routes.MgmtFuncRun, {
    prototypeId: funcId,
    componentId: props.componentId,
    viewId: "DEFAULT",
  });

  const { req, newChangeSetId } = await call.post({});

  dispatchedFunc.value = true;
  setTimeout(() => {
    dispatchedFunc.value = false;
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

const showResourceInput = ref(false);

// MGMT funcs
const MGMT_RUN_KEY = "latestMgmtFuncRuns";

const funcRunQuery = useQuery({
  queryKey: [ctx?.changeSetId, MGMT_RUN_KEY],
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

// The latest funcrun for this each mgmt prototype of this component, keyed bu the prototypeId
const latestFuncRuns = computed(() => {
  const runs = {} as Record<string, FuncRun>;

  if (!componentId.value) return runs;

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
        queryKey: [ctx?.changeSetId, MGMT_RUN_KEY],
      });
    }, 5000);
  }
});

onMounted(() => {
  keyEmitter.on("Escape", () => {
    close();
  });

  keyEmitter.on("KeyE", () => {
    if (!component.value?.toDelete) {
      eraseComponent();
    }
  });

  keyEmitter.on("Backspace", () => {
    if (!component.value?.toDelete) {
      deleteComponent();
    }
  });

  keyEmitter.on("KeyR", () => {
    if (component.value?.toDelete) {
      restoreComponent();
    }
  });

  keyEmitter.on("KeyU", () => {
    if (!component.value?.toDelete && isUpgradeable.value) {
      upgradeComponent();
    }
  });

  realtimeStore.subscribe(MGMT_RUN_KEY, `changeset/${ctx?.changeSetId.value}`, [
    {
      eventType: "FuncRunLogUpdated",
      callback: async (payload) => {
        if (mgmtFuncs.value.find((m) => m.funcId === payload.actionId)) {
          setTimeout(() => {
            queryClient.invalidateQueries({
              queryKey: [ctx?.changeSetId, MGMT_RUN_KEY],
            });
          }, 500);
        }
      },
    },
  ]);
});
onBeforeUnmount(() => {
  keyEmitter.off("Escape");
  keyEmitter.off("KeyE");
  keyEmitter.off("Backspace");
  keyEmitter.off("KeyR");
  keyEmitter.off("KeyU");
  realtimeStore.unsubscribe(MGMT_RUN_KEY);
});

const showBanner = computed(() => !component.value || component.value.toDelete);

const gridStateClass = computed(() => {
  let c;

  if (docsOpen.value) {
    c = "docs-open";
  } else {
    c = "docs-closed";
  }

  if (showBanner.value) {
    c += "-with-banner";
  } else {
    c += "-without-banner";
  }

  return c;
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
    close();
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
</script>

<style lang="less" scoped>
section.grid.docs-open-with-banner {
  grid-template-areas:
    "banner banner banner"
    "name name name"
    "attrs docs right";
  grid-template-rows: 2.5rem 2.5rem minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
}
section.grid.docs-closed-with-banner {
  grid-template-areas:
    "banner banner"
    "name name"
    "attrs right";
  grid-template-rows: 2.5rem 2.5rem minmax(0, 1fr);
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
.docs {
  grid-area: docs;
}
.right {
  grid-area: right;
}
.name {
  grid-area: name;
}
.attrs {
  grid-area: attrs;
}
.banner {
  grid-area: banner;
}
</style>
