<template>
  <div class="p-3 flex flex-col gap-2">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Add the names of the arguments to this function and their types.
    </h1>
    <div class="flex flex-row gap-1 items-center">
      <SiTextBox
        id="newArg"
        v-model="newArg.name"
        class="flex-1"
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
        class="flex-none"
        label="Add"
        button-rank="primary"
        button-type="success"
        :disabled="disabled"
        @click="addArgument"
      />
    </div>
    <ul>
      <li
        v-for="arg in editingArgs"
        :key="arg.id"
        class="flex flex-row items-center gap-1"
      >
        <SiTextBox
          :id="`arg-name-${arg.id}`"
          v-model="arg.name"
          class="flex-1"
          placeholder="Argument name"
          :disabled="disabled"
          @blur="saveArguments"
        />
        <SelectMenu
          v-model="arg.kind"
          class="flex-auto"
          :options="kindOptions"
          :disabled="disabled"
          @change="saveArguments"
        />
        <VButton
          class="flex-none"
          label="Del"
          button-rank="primary"
          button-type="destructive"
          :disabled="disabled"
          @click="deleteArgument(arg.name)"
        />
      </li>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { ref, toRef, watch } from "vue";
import { FuncArgument, FuncArgumentKind } from "@/api/sdf/dal/func";
import VButton from "@/molecules/VButton.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";

const funcStore = useFuncStore();

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
  funcId: string;
  arguments: FuncArgument[];
  disabled?: boolean;
}>();

const defaultNewArg = {
  id: -1,
  name: "",
  kind: kindToOption(FuncArgumentKind.String),
  elementKind: kindToOption(),
};

const funcId = toRef(props, "funcId", -1);
const args = toRef(props, "arguments", []);
const newArg = ref<EditingFuncArgument>(defaultNewArg);

interface EditingFuncArgument {
  id: number;
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

const editingArgs = ref<EditingFuncArgument[]>([]);

watch(
  args,
  (args) => {
    editingArgs.value = argsToEditingArgs(args);
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
  funcStore.updateFuncAttrArgs(
    funcId.value,
    editingArgsToArgs(editingArgs.value),
  );
};

const deleteArgument = async (name: string) => {
  editingArgs.value = editingArgs.value.filter((a) => a.name !== name);
  saveArguments();
};
</script>
