<template>
  <div>
    <div class="w-full flex p-2 gap-1 border-b dark:border-neutral-600">
      <VButton
        :disabled="disabled"
        tone="success"
        icon="plus"
        label="Add Binding"
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
        <h2 class="pb-2 text-sm">
          {{ proto.outputLocation?.label ?? "no output location set" }}
        </h2>

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
            tone="neutral"
            label="Edit Binding"
            size="md"
            @click="openModal(proto.id)"
          />
          <VButton
            :disabled="disabled"
            variant="transparent"
            tone="destructive"
            icon="x"
            label="Remove Binding"
            size="md"
            @click="removeBinding(proto.id)"
          />
        </div>
      </li>

      <AttributeBindingsModal
        :open="isModalOpen"
        :prototype="editingPrototype"
        :edit="editingPrototype !== undefined"
        type="save"
        @close="closeModal()"
        @save="saveModal"
      />
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { computed, inject, ref, Ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { VButton } from "@si/vue-lib/design-system";
import {
  AttributeAssocations,
  AttributePrototypeView,
} from "@/store/func/types";
import { FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { nilId } from "@/utils/nilId";
import AttributeBindingsModal from "./AttributeBindingsModal.vue";

const funcStore = useFuncStore();
const {
  internalProviderIdToSourceName,
  schemaVariantOptions,
  componentOptions,
  schemaVariantIdForAttributePrototype,
  outputLocationForAttributePrototype,
} = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: AttributeAssocations;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: AttributeAssocations): void;
  (e: "change", v: AttributeAssocations): void;
}>();

const associations = ref(props.modelValue);

watch(
  () => props.modelValue,
  (mv) => {
    associations.value = mv;
  },
  { immediate: true },
);

const editingPrototype = ref<AttributePrototypeView | undefined>(undefined);
const makeEmptyPrototype = (): AttributePrototypeView => ({
  id: nilId(),
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

const removeBinding = (prototypeId: string) => {
  associations.value.prototypes = associations.value.prototypes.filter(
    (proto) => proto.id !== prototypeId,
  );
  emit("update:modelValue", associations.value);
  emit("change", associations.value);
};

const addOrUpdateBinding = (
  associations: AttributeAssocations,
  prototype: AttributePrototypeView,
) => {
  if (prototype.id !== nilId()) {
    const currentPrototypeIdx = associations.prototypes.findIndex(
      (proto) => proto.id === prototype.id,
    );
    associations.prototypes[currentPrototypeIdx] = prototype;
  } else {
    associations.prototypes.push(prototype);
  }

  return associations;
};

const saveModal = (prototype?: AttributePrototypeView) => {
  if (prototype) {
    associations.value = addOrUpdateBinding(associations.value, prototype);
    emit("update:modelValue", associations.value);
    emit("change", associations.value);
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

const prototypeView = computed(() => {
  return associations.value.prototypes.map((proto) => {
    const schemaVariantId = schemaVariantIdForAttributePrototype.value?.(proto);
    const schemaVariant =
      schemaVariantOptions.value.find((sv) => sv.value === schemaVariantId)
        ?.label ?? "none";

    const component =
      componentOptions.value.find((c) => c.value === proto.componentId)
        ?.label ?? "all";

    const outputLocation = outputLocationForAttributePrototype.value?.(proto);

    const args = proto.prototypeArguments.map((arg) => ({
      name: funcArgumentsIdMap?.value[arg.funcArgumentId]?.name ?? "none",
      prop: arg.internalProviderId
        ? internalProviderIdToSourceName.value?.(arg.internalProviderId) ??
          "none"
        : "none",
    }));

    return {
      id: proto.id,
      schemaVariant,
      component,
      outputLocation,
      args,
    };
  });
});
</script>
