<template>
  <Modal
    :open="open"
    type="save"
    save-label="Save Bindings"
    size="2xl"
    @close="emit('close')"
    @save="emit('save', editedPrototype)"
  >
    <template #title
      >{{ props.prototype?.id === -1 ? "Add" : "Update" }} Function Bindings
    </template>
    <template #content>
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
          :options="componentOptions"
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
            <SelectMenu
              v-model="binding.binding"
              :options="inputSourceOptions"
            />
          </li>
        </ul>
      </div>
    </template>
  </Modal>
</template>

<script lang="ts" setup>
import { inject, watch, computed, toRef, ref, Ref } from "vue";
import Modal from "@/ui-lib/Modal.vue";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import { ListInputSourcesResponse } from "@/service/func/list_input_sources";
import { AttributePrototypeView } from "@/service/func";
import { LabelList } from "@/api/sdf/dal/label_list.js";
import { ComponentIdentification } from "@/api/sdf/dal/component.js";
import { FuncArgument } from "@/api/sdf/dal/func";

const props = withDefaults(
  defineProps<{
    open: boolean;
    funcId: number;
    prototype?: AttributePrototypeView;
  }>(),
  { open: false, edit: false },
);

const prototype = toRef(props, "prototype", undefined);

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", v?: AttributePrototypeView): void;
}>();

interface EditingBinding {
  id?: number;
  funcArgumentId: number;
  binding: Option;
}

const allComponentsOption = {
  label: "All components for schema variant",
  value: -1,
};
const noneVariant = { label: "select schema variant", value: -1 };
const noneOutputLocation = { label: "select place to store output", value: -1 };
const noneSource = { label: "select source", value: -1 };

const selectedVariant = ref<Option>(noneVariant);
const selectedComponent = ref<Option>(allComponentsOption);
const selectedOutputLocation = ref<Option>(noneOutputLocation);
const editableBindings = ref<EditingBinding[]>([]);

const inputSources = inject<Ref<ListInputSourcesResponse>>("inputSources");
const components =
  inject<Ref<LabelList<ComponentIdentification>>>("components");
const schemaVariantOptions = inject<Ref<Option[]>>("schemaVariantOptions");
const funcArgumentsIdMap =
  inject<Ref<{ [key: number]: FuncArgument }>>("funcArgumentsIdMap");

const editedPrototype = computed(() => ({
  id: props.prototype?.id ?? -1,
  schemaVariantId: selectedVariant.value.value as number,
  componentId: selectedComponent.value.value as number,
  propId: selectedOutputLocation.value.value as number,
  prototypeArguments: editableBindings.value.map(
    ({ id, funcArgumentId, binding }) => ({
      id: id ?? -1,
      funcArgumentId: funcArgumentId ?? -1,
      internalProviderId: binding.value as number,
    }),
  ),
}));

const componentOptions = computed<Option[]>(() =>
  [allComponentsOption].concat(
    components?.value
      .filter(
        (c) =>
          selectedVariant.value.value === -1 ||
          c.value.schemaVariantId === selectedVariant.value.value,
      )
      .map(({ label, value }) => ({
        label,
        value: value.componentId,
      })) ?? [],
  ),
);

const outputLocationOptions = computed<Option[]>(
  () =>
    inputSources?.value.props
      .filter(
        (prop) =>
          selectedVariant.value.value === -1 ||
          selectedVariant.value.value === prop.schemaVariantId,
      )
      .map((prop) => ({
        label: `${prop.path}${prop.name}`,
        value: prop.propId,
      })) ?? [],
);

const inputSourceOptions = computed<Option[]>(() => {
  const selectedVariantId = selectedVariant.value.value;
  const sockets =
    inputSources?.value.sockets
      .filter(
        (socket) =>
          (selectedVariantId === -1 ||
            selectedVariantId === socket.schemaVariantId) &&
          socket.internalProviderId,
      )
      .map((socket) => ({
        label: `Socket: ${socket.name}`,
        // internalProviderId will never be undefined given the condition above but the Typescript compiler
        // is not quite smart enough to figure that out.
        value: socket.internalProviderId ?? -1,
      })) ?? [];

  const props =
    inputSources?.value.props
      .filter(
        (prop) =>
          (selectedVariantId === -1 ||
            selectedVariantId === prop.schemaVariantId) &&
          prop.internalProviderId &&
          prop.propId !== selectedOutputLocation.value.value,
      )
      .map((prop) => ({
        label: `Attribute: ${prop.path}${prop.name}`,
        value: prop.internalProviderId ?? -1,
      })) ?? [];

  return sockets.concat(props);
});

// When variant changes, unset component if necessary
watch(
  () => selectedVariant.value,
  (selectedVariant, oldValue) => {
    const componentIdent = components?.value.find(
      (c) => c.value.componentId === selectedComponent.value.value,
    );

    if (componentIdent?.value.schemaVariantId !== selectedVariant.value) {
      selectedComponent.value = allComponentsOption;
    }

    // If we switched from another schema variant, unset selected output location
    if (oldValue.value !== -1) {
      selectedOutputLocation.value = noneOutputLocation;
    }
  },
);

// When component changes, ensure variant is set correctly
watch(
  () => selectedComponent.value,
  (selectedComponent) => {
    const componentIdent = components?.value.find(
      (c) => c.value.componentId === selectedComponent.value,
    );
    if (
      componentIdent &&
      selectedVariant.value.value !== componentIdent.value.schemaVariantId
    ) {
      selectedVariant.value =
        schemaVariantOptions?.value.find(
          (sv) => sv.value === componentIdent?.value.schemaVariantId,
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
      componentOptions.value.find(
        (c) => c.value === prototype.value?.componentId,
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
