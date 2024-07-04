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
      <template v-if="isCreating">
        <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
          Asset:
        </h1>
        <SelectMenu
          v-model="selectedVariant"
          class="flex-auto"
          :options="schemaVariantOptionsUnlocked"
          @change="variantChanged"
        />
      </template>
      <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Output location:
      </h1>
      <SelectMenu
        v-model="selectedOutputLocation"
        class="flex-auto"
        :disabled="!isCreating"
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
            {{ funcArgumentName(binding.funcArgumentId) ?? "none" }}
          </h1>
          <SelectMenu v-model="binding.binding" :options="inputSourceOptions" />
        </li>
      </ul>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import { computed, ref, ComputedRef } from "vue";
import { storeToRefs } from "pinia";
import { Modal, useModal } from "@si/vue-lib/design-system";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import {
  FuncArgumentId,
  Attribute,
  FuncBindingKind,
  FuncId,
  AttributePrototypeArgumentId,
} from "@/api/sdf/dal/func";
import {
  outputSocketsAndPropsFor,
  inputSocketsAndPropsFor,
} from "@/api/sdf/dal/schema";
import { useFuncStore } from "@/store/func/funcs.store";
import { useComponentsStore } from "@/store/components.store";
import { nilId } from "@/utils/nilId";

const componentsStore = useComponentsStore();
const { schemaVariantOptionsUnlocked, schemaVariantOptions } =
  storeToRefs(componentsStore);

const funcStore = useFuncStore();

const props = defineProps<{
  funcId: FuncId;
}>();

const bindingsModalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(bindingsModalRef);

const isCreating = ref(false);
const modalTitle = computed(
  () => `${isCreating.value ? "Add" : "Update"}  Function Bindings`,
);

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", v?: Attribute): void;
}>();

interface EditingBinding {
  id?: string;
  funcArgumentId: string;
  attributePrototypeArgumentId: AttributePrototypeArgumentId | null;
  binding: Option;
}

const noneOutputLocation = {
  label: "select place to store output",
  value: nilId(),
};
const noneSource = { label: "select source", value: nilId() };

const selectedOutputLocation = ref<Option>(noneOutputLocation);
const editableBindings = ref<EditingBinding[]>([]);

const openedWithBinding = ref<Attribute | null>(null);
const noneVariant = { label: "select schema variant", value: nilId() };
const selectedVariant = ref<Option>(noneVariant);

const funcArgumentName = (
  funcArgumentId: FuncArgumentId,
): string | undefined => {
  return funcStore.selectedFuncSummary?.arguments
    .filter((a) => a.id === funcArgumentId)
    .pop()?.name;
};

const editedPrototype: ComputedRef<Attribute> = computed(() => ({
  bindingKind: FuncBindingKind.Attribute,
  attributePrototypeId: openedWithBinding.value?.attributePrototypeId || null,
  componentId: null,
  funcId: props.funcId,
  schemaVariantId: selectedVariant.value?.value as string,
  propId: (selectedOutputLocation.value.value as string).startsWith("p_")
    ? (selectedOutputLocation.value.value as string).replace("p_", "")
    : null,
  outputSocketId: (selectedOutputLocation.value.value as string).startsWith(
    "s_",
  )
    ? (selectedOutputLocation.value.value as string).replace("s_", "")
    : null,
  argumentBindings: editableBindings.value.map(
    ({ funcArgumentId, attributePrototypeArgumentId, binding }) => ({
      funcArgumentId: funcArgumentId ?? null,
      inputSocketId: binding.label.includes("Input Socket")
        ? (binding.value as string).replace("s_", "")
        : null,
      propId: binding.label.includes("Attribute")
        ? (binding.value as string).replace("p_", "")
        : null,
      attributePrototypeArgumentId,
    }),
  ),
}));

const outputLocationOptions = computed(() => {
  let socketOptions: Option[] = [];
  let propOptions: Option[] = [];
  const variant =
    componentsStore.schemaVariantsById[selectedVariant.value.value as string];
  if (variant)
    ({ socketOptions, propOptions } = outputSocketsAndPropsFor(variant));

  return [...socketOptions, ...propOptions];
});

const inputSourceOptions = computed(() => {
  let socketOptions: Option[] = [];
  let propOptions: Option[] = [];
  const variant =
    componentsStore.schemaVariantsById[selectedVariant.value.value as string];
  if (variant)
    ({ socketOptions, propOptions } = inputSocketsAndPropsFor(variant));

  return [...socketOptions, ...propOptions];
});

const variantChanged = () => {
  selectedOutputLocation.value = noneOutputLocation;
};

// When prototype we're editing changes, set up defaults
const open = (binding: Attribute) => {
  isCreating.value = !binding.attributePrototypeId;
  openedWithBinding.value = binding;

  const startingVariant =
    componentsStore.schemaVariantsById[binding.schemaVariantId || ""];
  selectedVariant.value =
    schemaVariantOptions.value.find(
      (o) => o.value === startingVariant?.schemaVariantId,
    ) || noneVariant;

  selectedOutputLocation.value =
    outputLocationOptions.value.find(
      (loc) =>
        loc.value === `p_${binding.propId}` ||
        loc.value === `s_${binding.outputSocketId}`,
    ) || noneOutputLocation;

  editableBindings.value = [];
  if (binding.argumentBindings.length > 0) {
    editableBindings.value =
      binding?.argumentBindings.map(
        ({
          funcArgumentId,
          attributePrototypeArgumentId,
          inputSocketId,
          propId,
        }) => ({
          funcArgumentId,
          attributePrototypeArgumentId,
          binding:
            inputSourceOptions.value.find(
              (opt) =>
                opt.value === `s_${inputSocketId}` ||
                opt.value === `p_${propId}`,
            ) || noneSource,
        }),
      ) || [];
  } else {
    const funcArgs = funcStore.funcsById[props.funcId]?.arguments;
    if (funcArgs)
      editableBindings.value = funcArgs.map(({ id }) => ({
        funcArgumentId: id,
        attributePrototypeArgumentId: null,
        binding: noneSource,
      }));
  }

  openModal();
};

defineExpose({ open, close });
</script>
