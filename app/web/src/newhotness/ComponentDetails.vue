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
            'name flex flex-row items-center gap-xs p-xs ',
            themeClasses('bg-neutral-200', 'bg-neutral-800'),
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
      </div>

      <div class="attrs flex flex-col gap-sm">
        <CollapsingFlexItem ref="attrRef" :expandable="false" open>
          <template #header>
            <div class="flex place-content-between w-full">
              <span>Attributes</span>
              <template v-if="importFunc">
                <VButton
                  size="sm"
                  tone="neutral"
                  :label="
                    showResourceInput
                      ? 'Set attributes manually'
                      : 'Import a Resource'
                  "
                  class="ml-2xs mr-xs font-normal"
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
          <template #header>
            <PillCounter
              :count="(component.inputCount ?? 0) + (outgoing ?? 0)"
            />
            Connections
            <div class="ml-auto">
              <div
                :class="
                  clsx(
                    'text-sm cursor-pointer border px-xs py-2xs rounded',
                    'hover:text-action-500 hover:underline',
                    themeClasses(
                      'bg-neutral-400 border-neutral-300 text-black',
                      'bg-neutral-700 border-neutral-700 text-white',
                    ),
                  )
                "
                @click="navigateToMap"
              >
                See on Map
              </div>
            </div>
          </template>
          <ConnectionsPanel
            v-if="componentConnections && component"
            :component="component"
            :connections="componentConnections ?? undefined"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem open>
          <template #header>
            <PillCounter :count="component.qualificationTotals.total" />
            Qualifications
          </template>
          <QualificationPanel
            :component="component"
            :attributeTree="attributeTree"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem>
          <template #header>
            <Icon
              v-if="component.hasResource"
              name="check-hex"
              size="sm"
              tone="success"
            />
            <Icon v-else name="refresh-hex-outline" size="sm" tone="shade" />
            Resource
          </template>
          <ResourcePanel
            :component="component"
            :attributeTree="attributeTree ?? undefined"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem>
          <template #header>
            <Icon name="brackets-curly" size="sm" />
            Generated Code
          </template>
          <CodePanel
            v-if="attributeTree"
            :component="component"
            :attributeTree="attributeTree"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem>
          <template #header>
            <Icon name="tilde" size="sm" />
            Diff
          </template>
          <DiffPanel :component="component" />
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
  </section>
</template>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import {
  VButton,
  PillCounter,
  Icon,
  themeClasses,
  IconButton,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { computed, ref, onMounted, onBeforeUnmount, inject, watch } from "vue";
import { useRouter } from "vue-router";
import clsx from "clsx";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  AttributeTree,
  BifrostComponent,
  EntityKind,
  IncomingConnections,
} from "@/workers/types/entity_kind_types";
import { Context, assertIsDefined } from "@/newhotness/types";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import { useRealtimeStore } from "@/store/realtime/realtime.store";
import AttributePanel from "./AttributePanel.vue";
import ResourceValuesPanel from "./ResourceValuesPanel.vue";
import { attributeEmitter, keyEmitter } from "./logic_composables/emitters";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import DelayedLoader from "./layout_components/DelayedLoader.vue";
import { useApi, routes } from "./api_composables";
import { preserveExploreState } from "./util";
import { SelectionsInQueryString } from "./Workspace.vue";
import QualificationPanel from "./QualificationPanel.vue";
import ResourcePanel from "./ResourcePanel.vue";
import CodePanel from "./CodePanel.vue";
import DiffPanel from "./DiffPanel.vue";
import ActionsPanel from "./ActionsPanel.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";
import DocumentationPanel from "./DocumentationPanel.vue";
import ManagementPanel from "./ManagementPanel.vue";

const props = defineProps<{
  componentId: string;
}>();

const realtimeStore = useRealtimeStore();
const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const key = useMakeKey();
const args = useMakeArgs();
const queryClient = useQueryClient();

const docsOpen = ref(true);

const componentId = computed(() => props.componentId);

const outgoing = computed(
  () => ctx.outgoingCounts.value[props.componentId] ?? 0,
);

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

// TODO(Wendy) - this code is for if we want the AttributeInput to float again
// const scrollAttributePanel = (value: number) => {
//   if (attrRef.value) {
//     attrRef.value.setScroll(value);
//   }
// };

const router = useRouter();

const close = () => {
  const params = router.currentRoute?.value.params ?? {};
  delete params.componentId;
  router.push({
    name: "new-hotness",
    params,
    query: preserveExploreState(
      router.currentRoute.value?.query as SelectionsInQueryString,
    ),
  });
};

const navigateToMap = () => {
  const params = router.currentRoute?.value.params ?? {};
  delete params.componentId;
  const preservedQuery = preserveExploreState(
    router.currentRoute.value?.query as SelectionsInQueryString,
  );
  delete preservedQuery.grid;
  router.push({
    name: "new-hotness",
    params,
    query: {
      ...preservedQuery,
      map: "1",
      c: component.value?.id,
    },
  });
};

const api = useApi();

export type NameFormData = {
  name: string;
};

// Import
const importFunc = computed(() =>
  mgmtFuncs.value.find((f) => f.kind === "import"),
);

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
</script>

<style lang="less" scoped>
section.grid.docs-open-with-banner {
  grid-template-areas:
    "banner banner banner"
    "name docs right"
    "attrs docs right";
  grid-template-rows: 2.5rem 3rem minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
}
section.grid.docs-closed-with-banner {
  grid-template-areas:
    "banner banner"
    "name right"
    "attrs right";
  grid-template-rows: 2.5rem 3rem minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 33%);
}
section.grid.docs-open-without-banner {
  grid-template-areas:
    "name docs right"
    "attrs docs right";
  grid-template-rows: 3rem minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
}
section.grid.docs-closed-without-banner {
  grid-template-areas:
    "name right"
    "attrs right";
  grid-template-rows: 3rem minmax(0, 1fr);
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
