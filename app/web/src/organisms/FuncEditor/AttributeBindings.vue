<template>
  <div class="w-full flex p-2 gap-1 border-b dark:border-neutral-600">
    <VButton
      :disabled="disabled"
      button-rank="primary"
      button-type="success"
      icon="plus"
      label="Add binding"
      size="md"
      @click="openModal()"
    />
  </div>
  <ul class="flex flex-col p-3 gap-1">
    <li v-for="proto in prototypeView" :key="proto.id">
      <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Schema Variant:
      </h1>
      <h2 class="pb-2 text-sm">{{ proto.schemaVariant }}</h2>

      <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Component:
      </h1>
      <h2 class="pb-2 text-sm">{{ proto.component }}</h2>

      <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Output location:
      </h1>
      <h2 class="pb-2 text-sm">{{ proto.prop }}</h2>

      <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Expected Function Arguments:
      </h1>
      <h2 class="pb-2 text-sm">
        Below is the source of the data for each function argument listed.
      </h2>
      <ul>
        <li v-for="arg in proto.args" :key="arg.name">
          <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
            {{ arg.name }}
          </h1>
          <h2 class="pb-2 text-sm">{{ arg.prop }}</h2>
        </li>
      </ul>
      <div class="w-full flex p-2 gap-1 border-b dark:border-neutral-600">
        <VButton
          :disabled="disabled"
          button-rank="primary"
          button-type="neutral"
          label="Edit binding "
          size="md"
          @click="openModal(proto.id)"
        />
        <VButton
          :disabled="disabled"
          button-rank="tertiary"
          button-type="destructive"
          icon="x"
          label="Remove Binding"
          size="sm"
          @click="removeBinding(proto.id)"
        />
      </div>
    </li>
  </ul>
  <AttributeBindingsModal
    :func-id="funcId"
    :open="isModalOpen"
    :prototype="editingPrototype"
    :edit="editingPrototype !== undefined"
    type="save"
    @close="closeModal()"
    @save="saveModal"
  />
</template>

<script lang="ts" setup>
import { computed, inject, ref, Ref, toRef } from "vue";
import { storeToRefs } from "pinia";
import {
  AttributeAssocations,
  AttributePrototypeView,
} from "@/store/func/types";
import VButton from "@/molecules/VButton.vue";
import { FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import AttributeBindingsModal from "./AttributeBindingsModal.vue";

const funcStore = useFuncStore();
const {
  propIdToSourceName,
  providerIdToSourceName,
  schemaVariantOptions,
  componentOptions,
} = storeToRefs(funcStore);

function nilId(): string {
  return "00000000000000000000000000";
}

const editingPrototype = ref<AttributePrototypeView | undefined>(undefined);
const makeEmptyPrototype = (): AttributePrototypeView => ({
  id: nilId(),
  schemaVariantId: nilId(),
  componentId: nilId(),
  propId: nilId(),
  prototypeArguments: associations.value.arguments.map(({ id }) => ({
    funcArgumentId: id,
  })),
});

const isModalOpen = ref<boolean>(false);
const closeModal = () => {
  isModalOpen.value = false;
};

const removeBinding = (prototypeId?: string) =>
  prototypeId && funcStore.removeFuncAttrPrototype(props.funcId, prototypeId);

const saveModal = (prototype?: AttributePrototypeView) => {
  if (prototype) {
    funcStore.updateFuncAttrPrototype(props.funcId, prototype);
  }
  closeModal();
};

const openModal = (prototypeId?: string) => {
  // clear the prototype and then if we are editing an existing one, set it
  editingPrototype.value = makeEmptyPrototype();
  if (prototypeId) {
    editingPrototype.value = associations.value.prototypes.find(
      (proto) => proto.id === prototypeId,
    );
  }
  isModalOpen.value = true;
};

const funcArgumentsIdMap =
  inject<Ref<{ [key: string]: FuncArgument }>>("funcArgumentsIdMap");

const props = defineProps<{
  funcId: string;
  associations: AttributeAssocations;
  disabled?: boolean;
}>();

const associations = toRef(props, "associations", {
  type: "attribute",
  prototypes: [],
  arguments: [],
});

const prototypeView = computed(() => {
  console.log("prototypeView");
  return associations.value.prototypes.map((proto) => {
    const schemaVariant =
      schemaVariantOptions.value.find(
        (sv) => sv.value === proto.schemaVariantId,
      )?.label ?? "none";

    const component =
      componentOptions.value.find((c) => c.value === proto.componentId)
        ?.label ?? "all";
    const prop = propIdToSourceName.value[proto.propId] ?? "none";

    const args = proto.prototypeArguments.map((arg) => ({
      name: funcArgumentsIdMap?.value[arg.funcArgumentId]?.name ?? "none",
      prop: arg.internalProviderId
        ? providerIdToSourceName.value[arg.internalProviderId] ?? "none"
        : "none",
    }));

    return {
      id: proto.id,
      schemaVariant,
      component,
      prop,
      args,
    };
  });
});
</script>
