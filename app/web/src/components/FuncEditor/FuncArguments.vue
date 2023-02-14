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
        button-rank="primary"
        button-type="success"
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
          label="Del"
          button-rank="primary"
          button-type="destructive"
          :disabled="disabled"
          @click="deleteArgument(arg.name)"
        />
      </Inline>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { ref, toRef, watch } from "vue";
import { FuncArgument, FuncArgumentKind } from "@/api/sdf/dal/func";
import VButton from "@/components/VButton.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import Inline from "@/ui-lib/layout/Inline.vue";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
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

function nilId(): string {
  return "00000000000000000000000000";
}

const defaultNewArg = {
  id: nilId(),
  name: "",
  kind: kindToOption(FuncArgumentKind.String),
  elementKind: kindToOption(),
};

const funcId = toRef(props, "funcId", nilId());
const args = toRef(props, "arguments", []);
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
