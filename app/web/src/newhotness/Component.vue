<template>
  <section v-if="component" class="grid gap-md h-full p-md pb-0">
    <div class="left flex flex-col">
      <nav class="mb-md">
        <ol class="text-right [&>li]:cursor-pointer">
          <li>
            <VButton
              class="border-0"
              icon="arrow--left"
              size="sm"
              tone="shade"
              variant="ghost"
              label="Back"
              @click="back"
            />
          </li>
          <li @click="() => goto('attr')">Attributes</li>
          <li @click="() => goto('action')">Actions</li>
          <li @click="() => goto('mgmt')">Manangemnt Functions</li>
        </ol>
      </nav>
      <CollapsingFlexItem open>
        <template #header> Documentation </template>
      </CollapsingFlexItem>
    </div>
    <div class="name items-center flex flex-row gap-xs bg-gray-800 p-xs">
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
                  @input="(e) => field.handleChange((e.target as HTMLInputElement).value)"
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
    <div
      class="attrs scrollable flex flex-col"
      @scroll="() => attributeEmitter.emit('scrolled')"
    >
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
    <div class="right flex flex-col">
      <CollapsingFlexItem>
        <template #header>
          <PillCounter :count="component.inputCount + component.outputCount" />
          Connections
        </template>
        stuff
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
        <ResourcePanel
          v-if="component.rootAttributeValueId"
          :attributeValueId="component.rootAttributeValueId"
          :component="component"
        />
      </CollapsingFlexItem>
    </div>
  </section>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { VButton, PillCounter, Icon } from "@si/vue-lib/design-system";
import { computed, ref, nextTick } from "vue";
import { useRouter } from "vue-router";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import { BifrostComponent } from "@/workers/types/dbinterface";
import AttributePanel from "./AttributePanel.vue";
import { attributeEmitter } from "./logic_composables/emitters";
import CollapsingFlexItem from "./layout_components/CollapsingFlexItem.vue";
import EditInPlace from "./layout_components/EditInPlace.vue";
import { useApi, routes, UpdateComponentNameArgs } from "./api_composables";
import { useWatchedForm } from "./logic_composables/watched_form";
import QualificationPanel from "./QualificationPanel.vue";
import ResourcePanel from "./ResourcePanel.vue";
import { prevPage } from "./logic_composables/navigation_stack";

const props = defineProps<{
  componentId: string;
}>();

const componentQuery = useQuery<BifrostComponent | null>({
  queryKey: makeKey("Component", props.componentId),
  queryFn: async () => {
    return await bifrost<BifrostComponent>(
      makeArgs("Component", props.componentId),
    );
  },
});

const component = computed(() => componentQuery.data.value);

const attrRef = ref<typeof CollapsingFlexItem>();
const actionRef = ref<typeof CollapsingFlexItem>();
const mgmtRef = ref<typeof CollapsingFlexItem>();
const nameRef = ref<HTMLInputElement>();
const editInPlaceRef = ref<typeof EditInPlace>();

const goto = (target: "attr" | "action" | "mgmt") => {
  if (attrRef.value) attrRef.value.openState.open.value = target === "attr";
  if (actionRef.value)
    actionRef.value.openState.open.value = target === "action";
  if (mgmtRef.value) mgmtRef.value.openState.open.value = target === "mgmt";
};

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
  grid-template-columns: minmax(0, 25%) minmax(0, 1fr) minmax(0, 25%);
  grid-template-rows: 3rem minmax(0, 1fr);
  grid-template-areas:
    "left name right"
    "left attrs right";
}
.left {
  grid-area: left;
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
