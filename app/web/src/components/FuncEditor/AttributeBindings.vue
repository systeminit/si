<template>
  <div>
    <div
      v-if="!schemaVariantId"
      class="w-full flex p-xs gap-2xs border-b dark:border-neutral-600"
    >
      <VButton
        :disabled="disabled || variant?.isLocked"
        icon="plus"
        label="Add Binding"
        size="md"
        tone="success"
        @click="openModal()"
      />
    </div>
    <template v-if="bindings && bindings.length > 0">
      <ul class="flex flex-col p-3 gap-2xs break-words">
        <li v-for="bind in bindings" :key="bind.attributePrototypeId">
          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Asset:
          </h1>
          <h2 class="pb-xs text-sm">
            {{
              componentStore.schemaVariantsById[bind.schemaVariantId || ""]
                ?.displayName || "N/A"
            }}
          </h2>
          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Asset version:
          </h1>
          <h2 class="pb-xs text-sm">
            {{
              componentStore.schemaVariantsById[bind.schemaVariantId || ""]
                ?.version || "N/A"
            }}
          </h2>

          <!--<h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Component:
          </h1>
          <h2 class="pb-xs text-sm">{{ componentStore.componentsById[bind.componentId || ""]
                ?.displayName || "N/A" }}</h2>-->

          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Output location:
          </h1>
          <h2 class="pb-xs text-sm">
            {{ bind.outputDescription }}
          </h2>

          <h1 class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50">
            Expected Function Arguments:
          </h1>
          <h2 class="pb-xs text-sm">
            Below is the source of the data for each function argument listed.
          </h2>
          <ul>
            <li v-for="arg in bind.argumentBindings" :key="arg.funcArgumentId">
              <h1
                v-if="arg.propId"
                class="pt-xs text-neutral-700 type-bold-sm dark:text-neutral-50"
              >
                {{ getPropPathFrom(bind.schemaVariantId, arg.propId) }}
              </h1>
              <h2 v-if="arg.inputSocketId" class="pb-xs text-sm">
                {{ getSocketNameFrom(bind.schemaVariantId, arg.inputSocketId) }}
              </h2>
            </li>
          </ul>
          <div class="w-full flex p-xs gap-1 border-b dark:border-neutral-600">
            <VButton
              :disabled="disabled || bind.schemaVariant?.isLocked"
              label="Edit Binding"
              size="md"
              tone="neutral"
              @click="openModal(bind)"
            />
            <VButton
              :disabled="disabled || bind.schemaVariant?.isLocked"
              icon="x"
              label="Remove Binding"
              size="md"
              tone="destructive"
              variant="transparent"
              @click="removeBinding(bind)"
            />
          </div>
        </li>

        <AttributeBindingsModal
          ref="bindingsModalRef"
          :funcId="$props.funcId"
          :schemaVariantId="$props.schemaVariantId"
          type="save"
          @save="saveModal"
        />
      </ul>
    </template>
    <template v-else>
      <div v-if="$props.schemaVariantId">
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
  FuncId,
  Attribute,
  FuncBindingKind,
  AttributePrototypeId,
} from "@/api/sdf/dal/func";
import { PropId } from "@/api/sdf/dal/prop";
import {
  OutputSocketId,
  SchemaVariant,
  SchemaVariantId,
} from "@/api/sdf/dal/schema";
import { useFuncStore } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";
import { nonNullable } from "@/utils/typescriptLinter";
import AttributeBindingsModal from "./AttributeBindingsModal.vue";

const funcStore = useFuncStore();
const componentStore = useComponentsStore();

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId?: SchemaVariantId;
  disabled?: boolean;
}>();

const bindingsModalRef = ref<InstanceType<typeof AttributeBindingsModal>>();

const binding = computed(() => {
  if (!props.schemaVariantId) return null;

  const bindings = funcStore.attributeBindings[props.funcId];
  const binding = bindings
    ?.filter((b) => b.schemaVariantId === props.schemaVariantId)
    .pop();
  return binding;
});

const variant = computed(() => {
  return componentStore.schemaVariantsById[
    binding.value?.schemaVariantId || ""
  ];
});

const getPropPathFrom = (
  schemaVariantId: SchemaVariantId | null,
  propId: PropId,
) => {
  return componentStore.schemaVariantsById[schemaVariantId || ""]?.props.find(
    (p) => (p.id === propId ? p : null),
  )?.path;
};

const getSocketNameFrom = (
  schemaVariantId: SchemaVariantId | null,
  outputSocketId: OutputSocketId,
) => {
  return componentStore.schemaVariantsById[
    schemaVariantId || ""
  ]?.outputSockets.find((o) => (o.id === outputSocketId ? o : null))?.name;
};

interface ExtendedBinding extends Attribute {
  outputDescription: string;
  attributePrototypeId: AttributePrototypeId;
  schemaVariant: SchemaVariant | undefined;
}
const bindings = computed(() => {
  let b;
  if (props.schemaVariantId) b = [binding.value];
  else {
    b = funcStore.attributeBindings[props.funcId];
  }
  b = ((b as ExtendedBinding[]) || []).filter(nonNullable);
  return b.map((_b) => {
    _b.schemaVariant =
      componentStore.schemaVariantsById[_b.schemaVariantId || ""];
    if (_b.outputSocketId) {
      _b.outputDescription =
        getSocketNameFrom(_b.schemaVariantId, _b.outputSocketId) || "N/A";
    }
    if (_b.propId) {
      _b.outputDescription =
        getPropPathFrom(_b.schemaVariantId, _b.propId) || "N/A";
    }
    return _b;
  });
});

const makeBinding = () => {
  return {
    bindingKind: FuncBindingKind.Attribute,
    funcId: props.funcId,
    attributePrototypeId: null,
    schemaVariantId: props.schemaVariantId,
    componentId: null,
    propId: null,
    outputSocketId: null,
    argumentBindings: [],
  } as Attribute;
};

const removeBinding = async (binding: Attribute) => {
  await funcStore.RESET_ATTRIBUTE_BINDING(props.funcId, [binding]);
};

const addOrUpdateBinding = async (binding: Attribute) => {
  if (binding.attributePrototypeId) {
    await funcStore.UPDATE_BINDING(props.funcId, [binding]);
  } else {
    await funcStore.CREATE_BINDING(props.funcId, [binding]);
  }
};

const closeModal = () => {
  bindingsModalRef.value?.close();
};

const saveModal = (binding?: Attribute) => {
  if (binding) {
    addOrUpdateBinding(binding);
  }
  closeModal();
};

const openModal = (binding?: Attribute) => {
  if (!binding) binding = makeBinding();

  bindingsModalRef.value?.open(binding);
};

const detachFunc = async () => {
  if (binding.value)
    funcStore.RESET_ATTRIBUTE_BINDING(props.funcId, [binding.value]);
};

defineExpose({ detachFunc });
</script>
