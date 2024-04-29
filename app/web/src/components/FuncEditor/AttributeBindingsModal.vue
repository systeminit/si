<template>
  <Modal
    ref="bindingsModalRef"
    type="save"
    :saveLabel="isCreating ? 'Add Binding' : 'Update Binding'"
    size="2xl"
    :title="modalTitle"
    @save="emit('save', editedPrototype)"
  >
    <div class="p-4 flex flex-col place-content-center">
      <template v-if="!schemaVariantId">
        <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
          Schema Variant:
        </h1>
        <SelectMenu
          v-model="selectedVariant"
          class="flex-auto"
          :options="schemaVariantOptions ?? []"
        />
        <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
          Component:
        </h1>
        <SelectMenu
          v-model="selectedComponent"
          class="flex-auto"
          :options="filteredComponentOptions"
        />
      </template>
      <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Output location:
      </h1>
      <SelectMenu
        v-model="selectedOutputLocation"
        class="flex-auto"
        :options="outputLocationOptions"
      />
      <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Expected Function Arguments:
      </h1>
      <h2 class="pb-2 text-sm">
        Below is the source of the data for each function argument listed.
      </h2>
      <ul>
        <li v-for="binding in editableBindings" :key="binding.funcArgumentId">
          <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
            {{
              funcArgumentsIdMap
                ? funcArgumentsIdMap[binding.funcArgumentId]?.name ?? "none"
                : "none"
            }}
          </h1>
          <SelectMenu v-model="binding.binding" :options="inputSourceOptions" />
        </li>
      </ul>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import { inject, watch, computed, ref, Ref } from "vue";
import { storeToRefs } from "pinia";
import { Modal, useModal } from "@si/vue-lib/design-system";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { AttributePrototypeView, OutputLocation } from "@/store/func/types";
import { FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore, OutputLocationOption } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";

function nilId(): string {
  return "00000000000000000000000000";
}

const componentsStore = useComponentsStore();
const { allComponents } = storeToRefs(componentsStore);

const funcStore = useFuncStore();
const { schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  schemaVariantId?: string;
}>();

const bindingsModalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(bindingsModalRef);

const isCreating = ref(false);
const modalTitle = computed(
  () => `${isCreating.value ? "Add" : "Update"}  Function Bindings`,
);

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", v?: AttributePrototypeView): void;
}>();

interface EditingBinding {
  id?: string;
  funcArgumentId: string;
  binding: Option;
}

const allComponentsOption = {
  label: "All components for schema variant",
  value: nilId(),
};
const noneVariant = { label: "select schema variant", value: nilId() };
const noneOutputLocation = {
  label: "select place to store output",
  value: { label: "", propId: nilId() },
};
const noneSource = { label: "select source", value: nilId() };

const selectedVariant = ref<Option>(
  props.schemaVariantId
    ? { label: "", value: props.schemaVariantId }
    : noneVariant,
);
const selectedComponent = ref<Option>(allComponentsOption);
const selectedOutputLocation = ref<OutputLocationOption>(noneOutputLocation);
const editableBindings = ref<EditingBinding[]>([]);

const prototypeId = ref<string | undefined>();

const funcArgumentsIdMap =
  inject<Ref<{ [key: string]: FuncArgument }>>("funcArgumentsIdMap");

const editedPrototype = computed(() => ({
  id: prototypeId.value ?? nilId(),
  schemaVariantId: selectedVariant.value.value as string,
  componentId: selectedComponent.value.value as string,
  propId:
    "propId" in selectedOutputLocation.value.value
      ? selectedOutputLocation.value.value.propId
      : undefined,
  outputSocketId:
    "outputSocketId" in selectedOutputLocation.value.value
      ? selectedOutputLocation.value.value.outputSocketId
      : undefined,
  prototypeArguments: editableBindings.value.map(
    ({ id, funcArgumentId, binding }) => ({
      id: id ?? nilId(),
      funcArgumentId: funcArgumentId ?? nilId(),
      inputSocketId: binding.value as string,
    }),
  ),
}));

const filteredComponentOptions = computed<Option[]>(() =>
  [allComponentsOption].concat(
    allComponents.value
      .filter(
        (c) =>
          selectedVariant.value.value === nilId() ||
          c.schemaVariantId === selectedVariant.value.value,
      )
      .map(({ displayName, id }) => ({
        label: displayName,
        value: id,
      })) ?? [],
  ),
);

const outputLocationOptions = computed<
  { label: string; value: OutputLocation }[]
>(() =>
  funcStore.outputLocationOptionsForSchemaVariant(
    props.schemaVariantId ??
      (typeof selectedVariant.value.value === "string"
        ? selectedVariant.value.value
        : nilId()),
  ),
);

const inputSourceOptions = computed<Option[]>(() => {
  const selectedVariantId = selectedVariant.value.value as number;
  const socketOptions =
    funcStore.inputSourceSockets[selectedVariantId]?.map((socket) => ({
      label: `Input Socket: ${socket.name}`,
      value: socket.inputSocketId,
    })) ?? [];

  // TODO(nick): restore the true idea behind this. Likely requires more refactoring of prototype
  // arguments in the backend to support both inputSocketId and propId.
  const propOptions =
    funcStore.inputSourceProps[selectedVariantId]?.map((prop) => ({
      label: `Attribute: ${prop.path}${prop.name}`,
      value: nilId(),
    })) ?? [];

  return socketOptions.concat(propOptions);
});

// When variant changes, unset component if necessary
watch(selectedVariant, (selectedVariant) => {
  const componentIdent = allComponents.value.find(
    (c) => c.id === selectedComponent.value.value,
  );

  if (componentIdent?.schemaVariantId !== selectedVariant.value) {
    selectedComponent.value = allComponentsOption;
  }

  const currentSchemaVariantId = funcStore.schemaVariantIdForAttributePrototype(
    {
      id: nilId(),
      prototypeArguments: [],
      propId:
        "propId" in selectedOutputLocation.value.value
          ? selectedOutputLocation.value.value.propId
          : undefined,
      outputSocketId:
        "outputSocketId" in selectedOutputLocation.value.value
          ? selectedOutputLocation.value.value.outputSocketId
          : undefined,
    },
  );

  if ((selectedVariant.value as string) !== currentSchemaVariantId) {
    selectedOutputLocation.value = noneOutputLocation;
  }
});

// When component changes, ensure variant is set correctly
watch(
  selectedComponent,
  (selectedComponent) => {
    const componentIdent = allComponents.value.find(
      (c) => c.id === selectedComponent.value,
    );
    if (
      componentIdent &&
      selectedVariant.value.value !== componentIdent.schemaVariantId
    ) {
      selectedVariant.value =
        schemaVariantOptions?.value.find(
          (sv) => sv.value === componentIdent?.schemaVariantId,
        ) ?? noneVariant;
    }
  },
  { immediate: true },
);

// When prototype we're editing changes, set up defaults
const open = (prototype: AttributePrototypeView) => {
  const schemaVariantId =
    props.schemaVariantId ??
    funcStore.schemaVariantIdForAttributePrototype(prototype);

  prototypeId.value = prototype.id;

  selectedVariant.value =
    schemaVariantOptions?.value.find((sv) => sv.value === schemaVariantId) ??
    noneVariant;
  selectedComponent.value =
    filteredComponentOptions.value.find((c) => c.value === prototype.id) ??
    allComponentsOption;
  selectedOutputLocation.value =
    outputLocationOptions.value.find(
      (loc) =>
        ("propId" in loc.value && loc.value.propId === prototype.propId) ||
        ("outputSocketId" in loc.value &&
          loc.value.outputSocketId === prototype.outputSocketId),
    ) ?? noneOutputLocation;

  editableBindings.value =
    prototype?.prototypeArguments.map(
      ({ id, funcArgumentId, inputSocketId }) => ({
        id: id ?? undefined,
        funcArgumentId,
        binding:
          inputSourceOptions.value.find((opt) => opt.value === inputSocketId) ??
          noneSource,
      }),
    ) ?? [];

  openModal();
};

defineExpose({ open, close });
</script>
