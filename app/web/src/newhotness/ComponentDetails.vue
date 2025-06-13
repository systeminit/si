<template>
  <DelayedLoader v-if="componentQuery.isLoading.value" :size="'full'" />
  <section v-else-if="!component">
    <h3 class="text-destructive-500">
      This component does not exist on this change set
    </h3>
    <VButton
      v-tooltip="'Back (Esc)'"
      class="border-0 mr-2em"
      icon="arrow--left"
      size="sm"
      tone="shade"
      variant="ghost"
      @click="back"
    />
  </section>
  <!-- TODO(nick): replace this with a "read-only" view of the page and a banner -->
  <section v-else-if="component.toDelete">
    <h3 class="text-warning-500">
      This component has been marked for deletion.
    </h3>
    <VButton
      v-tooltip="'Back (Esc)'"
      class="border-0 mr-2em"
      icon="arrow--left"
      size="sm"
      tone="shade"
      variant="ghost"
      @click="back"
    />
  </section>
  <section
    v-else
    :class="
      clsx('grid gap-sm h-full p-sm', docsOpen ? 'docs-open' : 'docs-closed')
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
      <VButton
        v-tooltip="'Back (Esc)'"
        class="border-0 mr-2em flex-none"
        icon="arrow--left"
        size="sm"
        tone="shade"
        variant="ghost"
        @click="back"
      />
      <div class="flex-none">{{ component.schemaVariantName }}</div>
      <div class="flex-none">/</div>
      <div class="flex-1 min-w-0">
        <EditInPlace ref="editInPlaceRef" @hidden="reset" @showing="focus">
          <template #trigger>
            <VButton
              :label="component.name"
              :loading="wForm.bifrosting.value"
              :loadingText="component.name"
              class="border-0 font-normal max-w-full"
              iconRight="edit"
              loadingIcon="loader"
              size="sm"
              tone="shade"
              variant="ghost"
              truncateText
              @click="editInPlaceRef?.toggle"
            />
          </template>
          <template #input>
            <nameForm.Field
              :validators="{
                onChange: required,
                onBlur: required,
              }"
              name="name"
            >
              <template #default="{ field }">
                <input
                  ref="nameRef"
                  :value="field.state.value"
                  class="block w-full text-white bg-black border border-neutral-300 disabled:bg-neutral-900"
                  type="text"
                  @blur="blur"
                  @input="
                    (e) =>
                      field.handleChange((e.target as HTMLInputElement).value)
                  "
                  @keydown.enter.stop.prevent="blur"
                  @keydown.esc.stop.prevent="reset"
                />
              </template>
            </nameForm.Field>
          </template>
        </EditInPlace>
      </div>
    </div>

    <div class="attrs flex flex-col gap-sm">
      <CollapsingFlexItem ref="attrRef" :expandable="false" open>
        <template #header>
          <div class="flex place-content-between w-full">
            <span>Attributes</span>
            <template v-if="hasImportFunc">
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
          :showImportArea="showResourceInput"
        />
        <EmptyState v-else text="No attributes to display" icon="code-circle" />
      </CollapsingFlexItem>
      <CollapsingFlexItem ref="actionRef" :expandable="false">
        <template #header>Actions</template>
        <ActionsPanel
          :component="component"
          :attributeValueId="component.rootAttributeValueId"
        />
      </CollapsingFlexItem>
      <CollapsingFlexItem ref="mgmtRef" :expandable="false">
        <template #header>Management Functions</template>
        <ul v-if="mgmtFuncs.length > 0" class="p-xs flex flex-col gap-xs">
          <ManagementFuncCard
            v-for="func in mgmtFuncs"
            :key="func.id"
            :componentId="componentId"
            :func="func"
            :funcRun="latestFuncRuns[func.id]"
          />
        </ul>
        <EmptyState
          v-else
          text="No management functions available"
          icon="tools"
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
            :count="(component.inputCount ?? 0) + (component.outputCount ?? 0)"
          />
          Connections
        </template>
        <ConnectionsPanel
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
  </section>
</template>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { useInfiniteQuery, useQuery } from "@tanstack/vue-query";
import {
  VButton,
  PillCounter,
  Icon,
  themeClasses,
} from "@si/vue-lib/design-system";
import {
  computed,
  ref,
  nextTick,
  onMounted,
  onBeforeUnmount,
  inject,
} from "vue";
import { useRoute, useRouter } from "vue-router";
import clsx from "clsx";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  AttributeTree,
  BifrostComponent,
  BifrostComponentConnections,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { Context } from "@/newhotness/types";
import { FuncRun } from "@/newhotness/api_composables/func_run";
import ManagementFuncCard from "@/newhotness/ManagementFuncCard.vue";
import AttributePanel from "./AttributePanel.vue";
import { attributeEmitter, keyEmitter } from "./logic_composables/emitters";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import DelayedLoader from "./layout_components/DelayedLoader.vue";
import EditInPlace from "./layout_components/EditInPlace.vue";
import {
  useApi,
  routes,
  componentTypes,
  funcRunTypes,
} from "./api_composables";
import { useWatchedForm } from "./logic_composables/watched_form";
import QualificationPanel from "./QualificationPanel.vue";
import ResourcePanel from "./ResourcePanel.vue";
import CodePanel from "./CodePanel.vue";
import DiffPanel from "./DiffPanel.vue";
import ActionsPanel from "./ActionsPanel.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";
import DocumentationPanel from "./DocumentationPanel.vue";
import EmptyState from "./EmptyState.vue";

const props = defineProps<{
  componentId: string;
}>();

const docsOpen = ref(true);

const key = useMakeKey();
const args = useMakeArgs();

const componentId = computed(() => props.componentId);

const componentQuery = useQuery<BifrostComponent | null>({
  queryKey: key(EntityKind.Component, componentId),
  queryFn: async (queryContext) => {
    const component = await bifrost<BifrostComponent>(
      args(EntityKind.Component, componentId.value),
    );
    if (!component) {
      return queryContext.client.getQueryData(
        key(EntityKind.Component, componentId).value,
      ) as BifrostComponent | null;
    }
    return component;
  },
});

const attributeTreeQuery = useQuery<AttributeTree | null>({
  queryKey: key(EntityKind.AttributeTree, componentId.value),
  queryFn: async () => {
    const attributeTree = await bifrost<AttributeTree>(
      args(EntityKind.AttributeTree, componentId.value),
    );
    return attributeTree;
  },
});
const attributeTree = computed(() => attributeTreeQuery.data.value);

const component = computed(() => componentQuery.data.value);

const mgmtFuncs = computed(
  () => component.value?.schemaVariant.mgmtFunctions ?? [],
);

const componentConnectionsQuery = useQuery<BifrostComponentConnections | null>({
  queryKey: key(EntityKind.IncomingConnections, componentId),
  queryFn: async () => {
    const incomingConnections = await bifrost<BifrostComponentConnections>(
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
const nameRef = ref<HTMLInputElement>();
const editInPlaceRef = ref<typeof EditInPlace>();

// TODO(Wendy) - this code is for if we want the AttributeInput to float again
// const scrollAttributePanel = (value: number) => {
//   if (attrRef.value) {
//     attrRef.value.setScroll(value);
//   }
// };

const router = useRouter();

const back = () => {
  const params = router.currentRoute?.value.params ?? {};
  router.push({
    name: "new-hotness",
    params,
  });
};

const api = useApi();

export type NameFormData = {
  name: string;
};

const nameFormData = computed<NameFormData>(() => {
  return { name: component.value?.name ?? "" };
});

const wForm = useWatchedForm<NameFormData>(
  `component.name.${props.componentId}`,
);

const route = useRoute();

const nameForm = wForm.newForm({
  data: nameFormData,
  onSubmit: async ({ value }) => {
    const name = value.name;
    // i wish the validator narrowed this type to always be a string
    if (name) {
      const id = component.value?.id;
      if (!id) throw new Error("Missing id");
      const call = api.endpoint(routes.UpdateComponentName, { id });
      const { req, newChangeSetId } =
        await call.put<componentTypes.UpdateComponentNameArgs>({
          name,
        });
      if (newChangeSetId && api.ok(req)) {
        api.navigateToNewChangeSet(
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
    }
  },
});

const required = ({ value }: { value: string | undefined }) => {
  const len = value?.trim().length ?? 0;
  return len > 0 ? undefined : "Name required";
};

const reset = () => {
  if (nameForm.state.isSubmitted || nameForm.state.isDirty)
    nameForm.reset(nameFormData.value);
};

const focus = () => {
  nextTick(() => {
    if (nameRef.value) nameRef.value.focus();
  });
};

const blur = () => {
  if (nameForm.fieldInfo.name.instance?.state.meta.isDirty) {
    // don't double submit if you were `select()'d'`
    if (!nameForm.baseStore.state.isSubmitted) nameForm.handleSubmit();
  } else {
    reset();
  }

  if (editInPlaceRef.value) editInPlaceRef.value.hide();
};

// TODO(Wendy) - this code is for if we want the AttributeInput to float again
// type ScrollFunc = (value: number) => void;

// export type ComponentPageContext = {
//   scrollAttributePanel: ScrollFunc;
//   attributePanelScrollY: Ref<number>;
// };

// const context = reactive<ComponentPageContext>({
//   scrollAttributePanel,
//   attributePanelScrollY: ref(0),
// });

// watch(
//   () => attrRef.value?.scrollY,
//   () => {
//     if (attrRef.value) {
//       context.attributePanelScrollY = attrRef.value.scrollY;
//     }
//   },
// );

// provide("ComponentPageContext", context);

// Import
const hasImportFunc = computed(
  () => mgmtFuncs.value.find((f) => f.kind === "import") !== undefined,
);

const showResourceInput = ref(false);

// MGMT funcs
const ctx = inject<Context>("CONTEXT");

const funcRunQuery = useInfiniteQuery({
  queryKey: [ctx?.changeSetId, "paginatedFuncRuns"],
  queryFn: async ({
    pageParam = undefined,
  }): Promise<funcRunTypes.GetFuncRunsPaginatedResponse> => {
    const call = api.endpoint<funcRunTypes.GetFuncRunsPaginatedResponse>(
      routes.GetFuncRunsPaginated,
    );
    const params = new URLSearchParams();
    params.append("limit", "10");
    if (pageParam) {
      params.append("cursor", pageParam);
    }
    const req = await call.get(params);
    if (api.ok(req)) {
      return req.data;
    }
    return {
      funcRuns: [],
      nextCursor: null,
    };
  },
  initialPageParam: undefined,
  getNextPageParam: (lastPage: funcRunTypes.GetFuncRunsPaginatedResponse) => {
    return lastPage.nextCursor ?? undefined;
  },
});

const funcRuns = computed<FuncRun[]>(() => {
  if (!funcRunQuery.data.value) return [];
  return funcRunQuery.data.value.pages.flatMap((page) => page.funcRuns);
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

onMounted(() => {
  keyEmitter.on("Escape", () => {
    back();
  });
});
onBeforeUnmount(() => {
  keyEmitter.off("Escape");
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
