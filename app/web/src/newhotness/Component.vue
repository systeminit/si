<template>
  <section v-if="!component">
    <h3 class="text-destructive-500">
      This component does not exist on this change set
    </h3>
    <VButton
      class="border-0 mr-2em"
      icon="arrow--left"
      size="sm"
      tone="shade"
      variant="ghost"
      label="Back"
      @click="back"
    />
  </section>
  <section v-else class="grid gap-md h-full p-md pb-0">
    <div class="name items-center flex flex-row gap-xs bg-gray-800 p-xs">
      <VButton
        class="border-0 mr-2em"
        icon="arrow--left"
        size="sm"
        tone="shade"
        variant="ghost"
        label="Back"
        @click="back"
      />
      <span>{{ component.schemaVariantName }}</span>
      <span>/</span>
      <span class="grow">
        <EditInPlace ref="editInPlaceRef" @hidden="reset" @showing="focus">
          <template #trigger>
            <VButton
              class="border-0 font-normal"
              :label="component.name"
              size="sm"
              tone="shade"
              variant="ghost"
              iconRight="edit"
              loadingIcon="loader"
              :loading="wForm.bifrosting.value"
              :loadingText="component.name"
              @click="editInPlaceRef?.toggle"
            />
          </template>
          <template #input>
            <nameForm.Field
              name="name"
              :validators="{
                onChange: required,
                onBlur: required,
              }"
            >
              <template #default="{ field }">
                <input
                  ref="nameRef"
                  class="block w-full text-white bg-black border-2 border-neutral-300 disabled:bg-neutral-900"
                  type="text"
                  :value="field.state.value"
                  @input="
                    (e) =>
                      field.handleChange((e.target as HTMLInputElement).value)
                  "
                  @blur="blur"
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
        <AttributePanel
          v-if="component.rootAttributeValueId"
          :attributeValueId="component.rootAttributeValueId"
          :component="component"
        />
      </CollapsingFlexItem>
      <CollapsingFlexItem ref="actionRef" :expandable="false">
        <template #header>Actions</template>
        stuff
      </CollapsingFlexItem>
      <CollapsingFlexItem ref="mgmtRef" :expandable="false">
        <template #header>Management Functions</template>
        stuff
      </CollapsingFlexItem>
    </div>

    <div class="docs flex flex-col">
      <CollapsingFlexItem open>
        <template #header> Documentation </template>
        <template v-if="!docs">
          <p v-if="component.schemaVariantDocLink">
            <a :href="component.schemaVariantDocLink" target="_blank">{{
              component.schemaVariantName
            }}</a>
          </p>
          <p>{{ component.schemaVariantDescription }}</p>
        </template>
        <template v-else>
          <VButton
            class="border-0 mr-2em"
            icon="arrow--left"
            size="sm"
            tone="shade"
            variant="ghost"
            label="Back"
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
          <PillCounter :count="component.inputCount + component.outputCount" />
          Connections
        </template>
        {{ componentConnectionsPretty }}
      </CollapsingFlexItem>
      <CollapsingFlexItem open>
        <template #header>
          <PillCounter :count="component.qualificationTotals.total" />
          Qualifications
        </template>
        <QualificationPanel
          v-if="component.rootAttributeValueId"
          :attributeValueId="component.rootAttributeValueId"
          :component="component"
        />
      </CollapsingFlexItem>
      <CollapsingFlexItem h3class="flex flex-row items-center">
        <template #header>
          <Icon
            v-if="component.hasResource"
            name="check-hex"
            tone="success"
            size="sm"
          />
          <Icon v-else name="refresh-hex-outline" tone="shade" size="sm" />
          Resource
        </template>
        <ResourcePanel :attributeValueId="component.rootAttributeValueId" />
      </CollapsingFlexItem>
      <CollapsingFlexItem h3class="flex flex-row items-center">
        <template #header>
          <Icon name="brackets-curly" size="sm" />
          Generated Code
        </template>
        <CodePanel :attributeValueId="component.rootAttributeValueId" />
      </CollapsingFlexItem>
    </div>
  </section>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { VButton, PillCounter, Icon } from "@si/vue-lib/design-system";
import { computed, ref, nextTick } from "vue";
import { useRouter } from "vue-router";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostComponent,
  BifrostComponentConnectionsBeta,
} from "@/workers/types/dbinterface";
import AttributePanel from "./AttributePanel.vue";
import { attributeEmitter } from "./logic_composables/emitters";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import EditInPlace from "./layout_components/EditInPlace.vue";
import { useApi, routes, UpdateComponentNameArgs } from "./api_composables";
import { useWatchedForm } from "./logic_composables/watched_form";
import QualificationPanel from "./QualificationPanel.vue";
import ResourcePanel from "./ResourcePanel.vue";
import { prevPage } from "./logic_composables/navigation_stack";
import CodePanel from "./CodePanel.vue";

const props = defineProps<{
  componentId: string;
}>();

const componentId = computed(() => props.componentId);

const key = useMakeKey();
const args = useMakeArgs();

const componentQuery = useQuery<BifrostComponent | null>({
  queryKey: key("Component", componentId),
  queryFn: async () => {
    const component = await bifrost<BifrostComponent>(
      args("Component", componentId.value),
    );
    return component;
  },
});
const component = computed(() => componentQuery.data.value);

const componentConnectionsQuery =
  useQuery<BifrostComponentConnectionsBeta | null>({
    queryKey: key("ComponentConnectionsBeta", componentId),
    queryFn: async () => {
      const componentConnections =
        await bifrost<BifrostComponentConnectionsBeta>(
          args("ComponentConnectionsBeta", componentId.value),
        );
      return componentConnections;
    },
  });
const componentConnections = computed(
  () => componentConnectionsQuery.data.value,
);
const componentConnectionsPretty = computed(() => {
  if (!componentConnections.value) return "";
  return JSON.stringify(componentConnections.value, null, 2);
});

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

const router = useRouter();

const back = () => {
  const last = prevPage();
  if (last) router.push(last);
  else {
    const params = router.currentRoute?.value.params ?? {};
    router.push({
      name: "new-hotness",
      params,
    });
  }
};

const api = useApi();

type NameFormData = {
  name: string;
};

const nameFormData = computed<NameFormData>(() => {
  return { name: component.value?.name ?? "" };
});

const wForm = useWatchedForm<NameFormData>();

const nameForm = wForm.newForm(nameFormData, async ({ value }) => {
  const name = value.name;
  // i wish the validator narrowed this type to always be a string
  if (name) {
    const id = component.value?.id;
    if (!id) throw new Error("Missing id");
    const call = api.endpoint(routes.UpdateComponentName, { id });
    await call.put<UpdateComponentNameArgs>({ name });
  }
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
