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
import { inject, watch, computed, toRef, ref, Ref } from "vue";
import { storeToRefs } from "pinia";
import Modal from "@/ui-lib/modals/Modal.vue";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import { AttributePrototypeView } from "@/store/func/types";
import { FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";

function nilId(): string {
  return "00000000000000000000000000";
}

const componentsStore = useComponentsStore();
const { allComponents } = storeToRefs(componentsStore);

const funcStore = useFuncStore();
const { schemaVariantOptions, inputSources, propsAsOptionsForSchemaVariant } =
  storeToRefs(funcStore);

const props = withDefaults(
  defineProps<{
    open: boolean;
    funcId: string;
    prototype?: AttributePrototypeView;
  }>(),
  { open: false, edit: false },
);

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
  value: nilId(),
};
const noneSource = { label: "select source", value: nilId() };

const selectedVariant = ref<Option>(noneVariant);
const selectedComponent = ref<Option>(allComponentsOption);
const selectedOutputLocation = ref<Option>(noneOutputLocation);
const editableBindings = ref<EditingBinding[]>([]);

const funcArgumentsIdMap =
  inject<Ref<{ [key: string]: FuncArgument }>>("funcArgumentsIdMap");

const editedPrototype = computed(() => ({
  id: props.prototype?.id ?? nilId(),
  schemaVariantId: selectedVariant.value.value as string,
  componentId: selectedComponent.value.value as string,
  propId: selectedOutputLocation.value.value as string,
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

const outputLocationOptions = computed<Option[]>(() =>
  propsAsOptionsForSchemaVariant.value(
    typeof selectedVariant.value.value === "string"
      ? selectedVariant.value.value
      : nilId(),
  ),
);

const inputSourceOptions = computed<Option[]>(() => {
  const selectedVariantId = selectedVariant.value.value;
  const sockets =
    inputSources?.value.sockets
      .filter(
        (socket) =>
          (selectedVariantId === nilId() ||
            selectedVariantId === socket.schemaVariantId) &&
          socket.internalProviderId,
      )
      .map((socket) => ({
        label: `Socket: ${socket.name}`,
        // internalProviderId will never be undefined given the condition above but the Typescript compiler
        // is not quite smart enough to figure that out.
        value: socket.internalProviderId ?? nilId(),
      })) ?? [];

  const props =
    inputSources?.value.props
      .filter(
        (prop) =>
          (selectedVariantId === nilId() ||
            selectedVariantId === prop.schemaVariantId) &&
          prop.internalProviderId &&
          prop.propId !== selectedOutputLocation.value.value,
      )
      .map((prop) => ({
        label: `Attribute: ${prop.path}${prop.name}`,
        value: prop.internalProviderId ?? nilId(),
      })) ?? [];

  return sockets.concat(props);
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
    selectedVariant.value =
      schemaVariantOptions?.value.find(
        (sv) => sv.value === prototype.value?.schemaVariantId,
      ) ?? noneVariant;
    selectedComponent.value =
      filteredComponentOptions.value.find(
        (c) => c.value === prototype.value?.id,
      ) ?? allComponentsOption;
    selectedOutputLocation.value =
      outputLocationOptions.value.find(
        (loc) => loc.value === prototype.value?.propId,
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
