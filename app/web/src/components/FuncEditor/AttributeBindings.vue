<template>
  <div>
    <div
      v-if="!schemaVariantId"
      class="w-full flex p-2 gap-1 border-b dark:border-neutral-600"
    >
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
      <li v-for="proto in prototypeViews" :key="proto.id">
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
            v-if="!schemaVariantId"
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
        ref="bindingsModalRef"
        :schemaVariantId="schemaVariantId"
        type="save"
        @save="saveModal"
      />
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { computed, inject, ref, Ref, watch } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import {
  AttributeAssociations,
  AttributePrototypeView,
  FuncAssociations,
} from "@/store/func/types";
import { FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { nilId } from "@/utils/nilId";
import AttributeBindingsModal from "./AttributeBindingsModal.vue";

const funcStore = useFuncStore();

const props = defineProps<{
  modelValue: AttributeAssociations;
  schemaVariantId?: string;
  disabled?: boolean;
}>();

const bindingsModalRef = ref<InstanceType<typeof AttributeBindingsModal>>();

const emit = defineEmits<{
  (e: "update:modelValue", v: AttributeAssociations): void;
  (e: "change", v: AttributeAssociations): void;
}>();

const associations = ref(props.modelValue);

watch(
  () => props.modelValue,
  (mv) => {
    associations.value = mv;
  },
  { immediate: true },
);

const makeEmptyPrototype = (): AttributePrototypeView => ({
  id: nilId(),
  componentId: nilId(),
  propId: nilId(),
  prototypeArguments: associations.value.arguments.map(({ id }) => ({
    funcArgumentId: id,
  })),
});

const removeBinding = (prototypeId: string) => {
  associations.value.prototypes = associations.value.prototypes.filter(
    (proto) => proto.id !== prototypeId,
  );
  emit("update:modelValue", associations.value);
  emit("change", associations.value);
};

const addOrUpdateBinding = (
  associations: AttributeAssociations,
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

const closeModal = () => {
  bindingsModalRef.value?.close();
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
  const prototype = prototypeId
    ? associations.value.prototypes.find((proto) => proto.id === prototypeId)
    : makeEmptyPrototype();

  if (prototype) {
    bindingsModalRef.value?.open(prototype);
  }
};

const funcArgumentsIdMap =
  inject<Ref<{ [key: string]: FuncArgument }>>("funcArgumentsIdMap");

const prototypeViews = computed(() =>
  associations.value.prototypes
    .filter((proto) => {
      const schemaVariantId =
        funcStore.schemaVariantIdForAttributePrototype(proto);
      return (
        !props.schemaVariantId || schemaVariantId === props.schemaVariantId
      );
    })
    .map((proto) => {
      const schemaVariantId =
        funcStore.schemaVariantIdForAttributePrototype(proto);
      const schemaVariant =
        funcStore.schemaVariantOptions.find(
          (sv) => sv.value === schemaVariantId,
        )?.label ?? "none";

      const component =
        funcStore.componentOptions.find((c) => c.value === proto.componentId)
          ?.label ?? "all";

      const outputLocation =
        funcStore.outputLocationForAttributePrototype(proto);

      const args = proto.prototypeArguments.map((arg) => ({
        name: funcArgumentsIdMap?.value[arg.funcArgumentId]?.name ?? "none",
        prop: arg.internalProviderId
          ? funcStore.internalProviderIdToSourceName(arg.internalProviderId) ??
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
    }),
);

const detachFunc = (): FuncAssociations | undefined => {
  if (props.schemaVariantId) {
    return {
      ...associations.value,
      prototypes: associations.value.prototypes.filter(
        (proto) =>
          funcStore.schemaVariantIdForAttributePrototype(proto) !==
          props.schemaVariantId,
      ),
    };
  }
};

defineExpose({ detachFunc });
</script>
