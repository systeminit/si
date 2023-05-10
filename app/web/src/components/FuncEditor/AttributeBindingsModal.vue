<template>
  <Modal
    ref="bindingsModalRef"
    type="save"
    :save-label="isCreating ? 'Add Binding' : 'Update Binding'"
    size="2xl"
    :title="modalTitle"
    @close="emit('close')"
    @save="emit('save', editedPrototype)"
  >
    <div class="p-4 flex flex-col place-content-center">
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
import { inject, watch, computed, toRef, ref, Ref, PropType } from "vue";
import { storeToRefs } from "pinia";
import { Modal } from "@si/vue-lib/design-system";
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
const {
  schemaVariantOptions,
  inputSourceSockets,
  inputSourceProps,
  schemaVariantIdForAttributePrototype,
  outputLocationOptionsForSchemaVariant,
} = storeToRefs(funcStore);

const props = defineProps({
  open: { type: Boolean, default: false },
  prototype: { type: Object as PropType<AttributePrototypeView> },
});

const bindingsModalRef = ref<InstanceType<typeof Modal>>();

const prototype = toRef(props, "prototype", undefined);
const open = toRef(props, "open", false);

const isCreating = computed(() => props.prototype?.id === nilId());
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

const selectedVariant = ref<Option>(noneVariant);
const selectedComponent = ref<Option>(allComponentsOption);
const selectedOutputLocation = ref<OutputLocationOption>(noneOutputLocation);
const editableBindings = ref<EditingBinding[]>([]);

const funcArgumentsIdMap =
  inject<Ref<{ [key: string]: FuncArgument }>>("funcArgumentsIdMap");

const editedPrototype = computed(() => ({
  id: props.prototype?.id ?? nilId(),
  schemaVariantId: selectedVariant.value.value as string,
  componentId: selectedComponent.value.value as string,
  propId:
    "propId" in selectedOutputLocation.value.value
      ? selectedOutputLocation.value.value.propId
      : undefined,
  externalProviderId:
    "externalProviderId" in selectedOutputLocation.value.value
      ? selectedOutputLocation.value.value.externalProviderId
      : undefined,
  prototypeArguments: editableBindings.value.map(
    ({ id, funcArgumentId, binding }) => ({
      id: id ?? nilId(),
      funcArgumentId: funcArgumentId ?? nilId(),
      internalProviderId: binding.value as string,
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
  outputLocationOptionsForSchemaVariant.value(
    typeof selectedVariant.value.value === "string"
      ? selectedVariant.value.value
      : nilId(),
  ),
);

const inputSourceOptions = computed<Option[]>(() => {
  const selectedVariantId = selectedVariant.value.value;
  const socketOptions =
    inputSourceSockets.value
      .filter(
        (socket) =>
          (selectedVariantId === nilId() ||
            selectedVariantId === socket.schemaVariantId) &&
          socket.internalProviderId,
      )
      .map((socket) => ({
        label: `Input Socket: ${socket.name}`,
        // internalProviderId will never be undefined given the condition above but the Typescript compiler
        // is not quite smart enough to figure that out.
        value: socket.internalProviderId ?? nilId(),
      })) ?? [];

  const propOptions =
    inputSourceProps.value
      .filter(
        (prop) =>
          (selectedVariantId === nilId() ||
            selectedVariantId === prop.schemaVariantId) &&
          prop.internalProviderId &&
          ("propId" in selectedOutputLocation.value.value
            ? prop.propId !== selectedOutputLocation.value.value.propId
            : true),
      )
      .map((prop) => ({
        label: `Attribute: ${prop.path}${prop.name}`,
        value: prop.internalProviderId ?? nilId(),
      })) ?? [];

  return socketOptions.concat(propOptions);
});

watch(
  () => open.value,
  (open) => {
    if (open) {
      bindingsModalRef?.value?.open();
    } else {
      bindingsModalRef?.value?.close();
    }
  },
  { immediate: true },
);

// When variant changes, unset component if necessary
watch(
  () => selectedVariant.value,
  (selectedVariant, oldValue) => {
    const componentIdent = allComponents.value.find(
      (c) => c.id === selectedComponent.value.value,
    );

    if (componentIdent?.schemaVariantId !== selectedVariant.value) {
      selectedComponent.value = allComponentsOption;
    }

    // If we switched from another schema variant, unset selected output location
    if (oldValue.value !== nilId()) {
      selectedOutputLocation.value = noneOutputLocation;
    }
  },
);

// When component changes, ensure variant is set correctly
watch(
  () => selectedComponent.value,
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
watch(
  () => props.open,
  () => {
    if (!prototype.value) {
      return;
    }

    const schemaVariantId = schemaVariantIdForAttributePrototype.value?.(
      prototype.value,
    );

    selectedVariant.value =
      schemaVariantOptions?.value.find((sv) => sv.value === schemaVariantId) ??
      noneVariant;
    selectedComponent.value =
      filteredComponentOptions.value.find(
        (c) => c.value === prototype.value?.id,
      ) ?? allComponentsOption;
    selectedOutputLocation.value =
      outputLocationOptions.value.find(
        (loc) =>
          ("propId" in loc.value &&
            loc.value.propId === prototype.value?.propId) ||
          ("externalProviderId" in loc.value &&
            loc.value.externalProviderId ===
              prototype.value?.externalProviderId),
      ) ?? noneOutputLocation;

    editableBindings.value =
      prototype.value?.prototypeArguments.map(
        ({ id, funcArgumentId, internalProviderId }) => ({
          id: id ?? undefined,
          funcArgumentId,
          binding:
            inputSourceOptions.value.find(
              (opt) => opt.value === internalProviderId,
            ) ?? noneSource,
        }),
      ) ?? [];
  },
  { immediate: true },
);
</script>
