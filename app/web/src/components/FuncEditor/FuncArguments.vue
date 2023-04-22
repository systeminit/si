<template>
  <div class="p-3 flex flex-col gap-2">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Add the names of the arguments to this function and their types.
    </h1>
    <Inline align-y="center">
      <VormInput
        id="newArg"
        v-model="newArg.name"
        type="text"
        no-label
        placeholder="New argument name..."
        :disabled="disabled"
      />
      <SelectMenu
        v-model="newArg.kind"
        class="flex-auto"
        :options="kindOptions"
        :disabled="disabled"
      />
      <VButton
        label="Add"
        tone="success"
        :disabled="disabled"
        @click="addArgument"
      />
    </Inline>
    <ul>
      <Inline v-for="arg in editingArgs" :key="arg.id" align-y="center">
        <VormInput
          :id="`arg-name-${arg.id}`"
          v-model="arg.name"
          type="text"
          no-label
          placeholder="Argument name"
          :disabled="disabled"
          @blur="saveArguments"
        />
        <SelectMenu
          v-model="arg.kind"
          :options="kindOptions"
          :disabled="disabled"
          @change="saveArguments"
        />
        <VButton
          label="Delete"
          tone="destructive"
          :disabled="disabled"
          @click="deleteArgument(arg.name)"
        />
      </Inline>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { Inline, VButton, VormInput } from "@si/vue-lib/design-system";
import { FuncArgument, FuncArgumentKind } from "@/api/sdf/dal/func";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { AttributeAssocations } from "@/store/func/types";
import { nilId } from "@/utils/nilId";

const generateKindOptions = () => {
  const options: Option[] = [];
  for (const kind in FuncArgumentKind) {
    options.push({ label: kind, value: kind });
  }
  return options;
};

const kindToOption = (kind?: FuncArgumentKind): Option =>
  kind
    ? { label: kind as string, value: kind as string }
    : { label: "None", value: 0 };

const kindOptions = generateKindOptions();
// we haven't implemented element kinds yet
// const elementKindOptions = [kindToOption()].concat(generateKindOptions());

const props = defineProps<{
  modelValue: AttributeAssocations;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: AttributeAssocations): void;
  (e: "change", v: AttributeAssocations): void;
}>();

const defaultNewArg = {
  id: nilId(),
  name: "",
  kind: kindToOption(FuncArgumentKind.String),
  elementKind: kindToOption(),
};

const newArg = ref<EditingFuncArgument>(defaultNewArg);

interface EditingFuncArgument {
  id: string;
  name: string;
  kind: Option;
  elementKind?: Option;
}

const argsToEditingArgs = (args: FuncArgument[]) =>
  args.map(({ id, name, kind, elementKind }) => ({
    id,
    name,
    kind: kindToOption(kind),
    elementKind: kindToOption(elementKind),
  }));

const editingArgsToArgs = (editingArgs: EditingFuncArgument[]) =>
  editingArgs.map(({ id, name, kind, elementKind }) => ({
    id,
    name,
    kind: kind.value as FuncArgumentKind,
    elementKind: elementKind?.value
      ? (elementKind.value as FuncArgumentKind)
      : undefined,
  }));

const associations = ref(props.modelValue);
const editingArgs = ref<EditingFuncArgument[]>(
  argsToEditingArgs(props.modelValue.arguments),
);

watch(
  () => props.modelValue,
  (mv) => {
    associations.value = mv;
    editingArgs.value = argsToEditingArgs(associations.value.arguments);
  },
  { immediate: true },
);

const addArgument = async () => {
  editingArgs.value.push({ ...newArg.value });
  newArg.value.name = "";
  newArg.value.kind = kindToOption(FuncArgumentKind.String);

  saveArguments();
};

const saveArguments = () => {
  associations.value.arguments = editingArgsToArgs(editingArgs.value);
  emit("change", associations.value);
  emit("update:modelValue", associations.value);
};

const deleteArgument = async (name: string) => {
  editingArgs.value = editingArgs.value.filter((a) => a.name !== name);
  saveArguments();
};
</script>
