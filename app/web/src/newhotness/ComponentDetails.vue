<template>
  <DelayedLoader v-if="componentQuery.isLoading.value" :size="'full'" />
  <section
    v-else
    :class="
      clsx(
        component && !component.toDelete && 'grid',
        'gap-sm h-full p-sm',
        docsOpen ? 'docs-open' : 'docs-closed',
      )
    "
  >
    <div
      :class="
        clsx(
          'name flex flex-row items-center gap-xs p-xs',
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

      <div v-if="!component" class="text-destructive-500">
        This component does not exist on this change set.
      </div>
      <!-- TODO(nick): replace this with a "read-only" view of the page and a banner -->
      <div v-else-if="component.toDelete" class="text-warning-500">
        This component has been marked for deletion.
      </div>
      <template v-else>
        <div class="flex-none">{{ component.schemaVariantName }}</div>
        <div class="flex-none">/</div>
        <div class="flex-1 min-w-0 m-[-4px]">{{ component.name }}</div>
      </template>
    </div>

    <template v-if="component && !component.toDelete">
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
            v-if="attributeTree"
            :component="component"
            :attributeTree="attributeTree"
            :importFunc="showResourceInput ? importFunc : undefined"
            :importFuncRun="latestFuncRuns[importFunc?.id ?? '']"
          />
          <EmptyState
            v-else
            text="No attributes to display"
            icon="code-circle"
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
          </template>
          <ConnectionsPanel
            v-if="componentConnections && component"
            :component="component"
            :connections="componentConnections ?? undefined"
          />
        </CollapsingFlexItem>
        <CollapsingFlexItem>
          <template #header>
            <PillCounter :count="component.qualificationTotals.total" />
            Qualifications
          </template>
          <QualificationPanel
            v-if="attributeTree"
            :component="component"
            :attributeTree="attributeTree"
          />
          <EmptyState
            v-else
            icon="question-circle"
            text="No qualifications to display"
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
import EmptyState from "./EmptyState.vue";
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
</script>

<style lang="less" scoped>
section.grid {
  grid-template-rows: 3rem minmax(0, 1fr);
}
section.grid.docs-open {
  grid-template-areas:
    "name docs right"
    "attrs docs right";
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
}
section.grid.docs-closed {
  grid-template-areas:
    "name right"
    "attrs right";
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
</style>
