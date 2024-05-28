<template>
  <div>
    <div
      v-if="!schemaVariantId"
      class="w-full flex p-xs gap-1 border-b dark:border-neutral-600"
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
            <h2 class="pb-2 text-sm">{{ arg.path }}</h2>
          </li>
        </ul>
        <div class="w-full flex p-xs gap-1 border-b dark:border-neutral-600">
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
import { computed, ref } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import {
  AttributeAssociations,
  AttributePrototypeBag,
  FuncAssociations,
} from "@/store/func/types";
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

const associations = computed(
  () =>
    funcStore.funcDetailsById[funcId.value as string]
      ?.associations as AttributeAssociations,
);

const funcArguments = computed(() => funcStore.funcArguments);
const funcId = computed(() => funcStore.selectedFuncId);

const makeEmptyPrototype = (): AttributePrototypeBag => ({
  id: nilId(),
  componentId: nilId(),
  propId: nilId(),
  prototypeArguments: funcArguments.value
    ? funcArguments.value.map(({ id }) => ({
        funcArgumentId: id,
      }))
    : [],
});

const rehydratePrototype = (
  existing: AttributePrototypeBag,
): AttributePrototypeBag => ({
  id: existing.id,
  componentId: existing.componentId,
  propId: existing.propId,
  prototypeArguments: existing.prototypeArguments,
});

const removeBinding = async (prototypeId: string) => {
  await funcStore.REMOVE_ATTRIBUTE_PROTOTYPE(prototypeId);
};

const addOrUpdateBinding = async (prototype: AttributePrototypeBag) => {
  if (prototype.id !== nilId()) {
    // update prototype
    await funcStore.UPDATE_ATTRIBUTE_PROTOTYPE(
      funcId.value as string,
      prototype.id as string,
      prototype.prototypeArguments,
      prototype.propId,
      prototype.outputSocketId,
    );
  } else {
    // create new prototype
    await funcStore.CREATE_ATTRIBUTE_PROTOTYPE(
      funcId.value as string,
      prototype.schemaVariantId as string,
      prototype.prototypeArguments,
      prototype.componentId,
      prototype.propId,
      prototype.outputSocketId,
    );
  }
};

const closeModal = () => {
  bindingsModalRef.value?.close();
};

const saveModal = (prototype?: AttributePrototypeBag) => {
  if (prototype) {
    addOrUpdateBinding(prototype);
  }
  closeModal();
};

const openModal = (prototypeId?: string) => {
  const prototype = prototypeId
    ? associations.value.prototypes.find((proto) => proto.id === prototypeId)
    : makeEmptyPrototype();

  if (prototype) {
    const proto = rehydratePrototype(prototype);
    bindingsModalRef.value?.open(proto);
  }
};

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
        name: funcStore.funcArgumentsById[arg.funcArgumentId]?.name ?? "none",
        path:
          funcStore.propIdToSourceName(arg.propId ?? nilId()) ??
          funcStore.inputSocketIdToSourceName(arg.inputSocketId ?? nilId()) ??
          "none",
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
    const prototype = associations.value.prototypes.find(
      (proto) =>
        funcStore.schemaVariantIdForAttributePrototype(proto) ===
        props.schemaVariantId,
    );
    // todo: remove the binding when the user hits the detach button
    removeBinding(prototype?.id as string);
    return;
  }
};

defineExpose({ detachFunc });
</script>
