<template>
  <div>
    <div
      v-if="!schemaVariantId"
      class="w-full flex p-xs gap-2xs border-b dark:border-neutral-600"
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
    <template v-if="prototypeViews.length > 0">
      <ul class="flex flex-col p-3 gap-2xs break-words">
        <li v-for="proto in prototypeViews" :key="proto.id">
          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Asset:
          </h1>
          <h2 class="pb-xs text-sm">{{ proto.schema }}</h2>
          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Asset version:
          </h1>
          <h2 class="pb-xs text-sm">{{ proto.schemaVariant }}</h2>

          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Component:
          </h1>
          <h2 class="pb-xs text-sm">{{ proto.component }}</h2>

          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Output location:
          </h1>
          <h2 class="pb-xs text-sm">
            {{ proto.outputLocation?.label ?? "no output location set" }}
          </h2>

          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Expected Function Arguments:
          </h1>
          <h2 class="pb-xs text-sm">
            Below is the source of the data for each function argument listed.
          </h2>
          <ul>
            <li v-for="arg in proto.args" :key="arg.name">
              <h1
                class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50"
              >
                {{ arg.name }}
              </h1>
              <h2 class="pb-xs text-sm">{{ arg.path }}</h2>
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
    </template>
    <template v-else>
      <div v-if="schemaVariantId">
        <p class="text-neutral-400 dark:text-neutral-300 text-sm p-xs">
          This function is not attached to this schema variant. Use the Attach
          Existing functionality to re-attach it.
        </p>
      </div>
      <div v-else></div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import {
  AttributeAssociations,
  AttributePrototypeBag,
} from "@/store/func/types";
import { useFuncStore } from "@/store/func/funcs.store";
import { nilId } from "@/utils/nilId";
import { useComponentsStore } from "@/store/components.store";
import AttributeBindingsModal from "./AttributeBindingsModal.vue";

const funcStore = useFuncStore();
const componentStore = useComponentsStore();

const props = defineProps<{
  modelValue: AttributeAssociations;
  schemaVariantId?: string;
  disabled?: boolean;
}>();

const bindingsModalRef = ref<InstanceType<typeof AttributeBindingsModal>>();

const funcArguments = computed(() => funcStore.funcArguments);
const funcId = computed(() => funcStore.selectedFuncId);

const associations = computed(
  () =>
    funcStore.funcDetailsById[funcId.value as string]
      ?.associations as AttributeAssociations,
);

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
  prototypeArguments: funcArguments.value
    ? funcArguments.value.map(({ id }) => {
        const foundArg = existing.prototypeArguments.find(
          (protoArg) => protoArg.funcArgumentId === id,
        );
        if (foundArg) {
          return {
            id: foundArg.id ?? nilId(),
            funcArgumentId: id,
            propId: foundArg.propId,
            inputSocketId: foundArg.inputSocketId,
          };
        }
        return { funcArgumentId: id };
      })
    : [],
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

const prototypeViews = computed(() => {
  const validPrototypes = associations.value.prototypes.filter((proto) => {
    // If no sv id on component, don't filter at all
    if (props.schemaVariantId === undefined) {
      return true;
    }

    const schemaVariantId =
      funcStore.schemaVariantIdForPrototypeTargetId[
        proto.propId ?? proto.outputSocketId ?? ""
      ];

    return schemaVariantId === props.schemaVariantId;
  });

  return validPrototypes.map((proto) => {
    const schemaVariantId =
      funcStore.schemaVariantIdForPrototypeTargetId[
        proto.propId ?? proto.outputSocketId ?? ""
      ];

    const schemaVariant =
      componentStore.schemaVariantsById[schemaVariantId ?? ""];
    const name = schemaVariant?.displayName || "none";
    const schema = schemaVariant?.schemaName || "none";
    const component =
      funcStore.componentOptions.find((c) => c.value === proto.componentId)
        ?.label ?? "all";

    const outputLocation = funcStore.outputLocationForAttributePrototype(proto);
    const args = funcStore.funcArguments?.map((funcArg) => ({
      name: funcArg.name,
      path: (() => {
        const protoArg = proto.prototypeArguments.find(
          (protoArg) => protoArg.funcArgumentId === funcArg.id,
        );
        if (protoArg) {
          return (
            funcStore.propIdToSourceName(protoArg.propId ?? nilId()) ??
            funcStore.inputSocketIdToSourceName(
              protoArg.inputSocketId ?? nilId(),
            ) ??
            "none"
          );
        }
        return "none";
      })(),
    }));

    return {
      id: proto.id,
      schema,
      schemaVariant: name,
      component,
      outputLocation,
      args,
    };
  });
});

const detachFunc = async (): Promise<undefined> => {
  if (props.schemaVariantId) {
    const prototype = associations.value.prototypes.find(
      (proto) =>
        funcStore.schemaVariantIdForPrototypeTargetId[
          proto.propId ?? proto.outputSocketId ?? ""
        ] === props.schemaVariantId,
    );
    // todo: remove the binding when the user hits the detach button
    await removeBinding(prototype?.id as string);
    return;
  }
};

defineExpose({ detachFunc });
</script>
