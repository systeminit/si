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
  <section v-else class="grid gap-md h-full p-md pb-0">
    <div
      :class="
        clsx(
          'name items-center flex flex-row gap-xs p-xs',
          themeClasses('bg-neutral-100', 'bg-neutral-900'),
        )
      "
    >
      <VButton
        v-tooltip="'Back (Esc)'"
        class="border-0 mr-2em"
        icon="arrow--left"
        size="sm"
        tone="shade"
        variant="ghost"
        @click="back"
      />
      <span>{{ component.schemaVariantName }}</span>
      <span>/</span>
      <span class="grow">
        <EditInPlace ref="editInPlaceRef" @hidden="reset" @showing="focus">
          <template #trigger>
            <VButton
              :label="component.name"
              :loading="wForm.bifrosting.value"
              :loadingText="component.name"
              class="border-0 font-normal"
              iconRight="edit"
              loadingIcon="loader"
              size="sm"
              tone="shade"
              variant="ghost"
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
      </span>
    </div>
    <div class="attrs flex flex-col">
      <CollapsingFlexItem ref="attrRef" :expandable="false" open>
        <template #header>Attributes</template>
        <AttributePanel :component="component" />
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
        <ul class="p-xs">
          <li
            v-for="func in mgmtFuncs"
            :key="func.id"
            class="rounded border border-neutral-600 p-xs h-12 flex flex-row items-center"
          >
            <IconButton disabled size="md" icon="play" iconTone="action" />
            {{ func.prototypeName }} {{ func }}
          </li>
        </ul>
      </CollapsingFlexItem>
    </div>

    <div class="docs flex flex-col">
      <CollapsingFlexItem open>
        <template #header> Documentation </template>
        <template v-if="!docs">
          <p v-if="component.schemaVariantDocLink">
            <a
              :href="component.schemaVariantDocLink"
              target="_blank"
              tabindex="-1"
              >{{ component.schemaVariantName }}</a
            >
          </p>
          <p>
            <VueMarkdown :source="component.schemaVariantDescription ?? ''" />
          </p>
        </template>
        <template v-else>
          <VButton
            class="border-0 mr-2em"
            icon="arrow--left"
            label="Back"
            size="sm"
            tone="shade"
            variant="ghost"
            @click="() => (docs = '')"
          />
          <p v-if="docLink">
            <a :href="docLink" target="_blank">{{
              component.schemaVariantName
            }}</a>
          </p>
          <p>{{ docs }}</p>
        </template>
      </CollapsingFlexItem>
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
          v-if="componentConnections"
          :component="component"
          :connections="componentConnections"
        />
      </CollapsingFlexItem>
      <CollapsingFlexItem open>
        <template #header>
          <PillCounter :count="component.qualificationTotals.total" />
          Qualifications
        </template>
        <QualificationPanel :component="component" />
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
        <ResourcePanel :component="component" />
      </CollapsingFlexItem>
      <CollapsingFlexItem>
        <template #header>
          <Icon name="brackets-curly" size="sm" />
          Generated Code
        </template>
        <CodePanel :component="component" />
      </CollapsingFlexItem>
      <CollapsingFlexItem>
        <template #header>
          <Icon name="tilde" size="sm" />
          Diff
        </template>
        <DiffPanel :component="component" />
      </CollapsingFlexItem>
    </div>
  </section>
</template>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import {
  VButton,
  PillCounter,
  Icon,
  IconButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { computed, ref, nextTick, onMounted, onBeforeUnmount } from "vue";
import { useRoute, useRouter } from "vue-router";
import VueMarkdown from "vue-markdown-render";
import clsx from "clsx";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostComponent,
  BifrostComponentConnections,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import AttributePanel from "./AttributePanel.vue";
import { attributeEmitter, keyEmitter } from "./logic_composables/emitters";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import DelayedLoader from "./layout_components/DelayedLoader.vue";
import EditInPlace from "./layout_components/EditInPlace.vue";
import { useApi, routes, componentTypes } from "./api_composables";
import { useWatchedForm } from "./logic_composables/watched_form";
import QualificationPanel from "./QualificationPanel.vue";
import ResourcePanel from "./ResourcePanel.vue";
import CodePanel from "./CodePanel.vue";
import DiffPanel from "./DiffPanel.vue";
import ActionsPanel from "./ActionsPanel.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";

const props = defineProps<{
  componentId: string;
}>();

const componentId = computed(() => props.componentId);

const key = useMakeKey();
const args = useMakeArgs();

const componentQuery = useQuery<BifrostComponent | null>({
  queryKey: key(EntityKind.Component, componentId),
  queryFn: async () => {
    const component = await bifrost<BifrostComponent>(
      args(EntityKind.Component, componentId.value),
    );
    return component;
  },
});
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

type NameFormData = {
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
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
  grid-template-rows: 3rem minmax(0, 1fr);
  grid-template-areas:
    "name docs right"
    "attrs docs right";
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
