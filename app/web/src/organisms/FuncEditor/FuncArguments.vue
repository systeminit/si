<template>
  <div>
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
        v-for="arg in args"
        :key="`${arg.name}-${arg.id}`"
        class="flex flex-row items-center gap-1"
      >
        <SiTextBox
          :id="`${arg.name}-${arg.id}`"
          v-model="arg.name"
          class="flex-1"
          placeholder="Argument name"
          :disabled="disabled"
        />
        <SelectMenu
          v-model="arg.kind"
          class="flex-auto"
          :options="kindOptions"
          :disabled="disabled"
        />
      </li>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { ref, toRef, watch } from "vue";
import { FuncArgument, FuncArgumentKind } from "@/api/sdf/dal/func";
import { FuncService } from "@/service/func.js";
import VButton from "@/molecules/VButton.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";

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
    : { label: "None", value: "None" };

const kindOptions = generateKindOptions();
// const elementKindOptions = [kindToOption()].concat(generateKindOptions());

const props = defineProps<{
  funcId: number;
  disabled?: boolean;
}>();

const defaultNewArg = {
  id: -1,
  name: "",
  kind: kindToOption(FuncArgumentKind.String),
  elementKind: kindToOption(),
};

const funcId = toRef(props, "funcId", -1);
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

const args = ref<EditingFuncArgument[]>([]);

const grabArgs = async (funcId: number) => {
  const funcArgs = (await FuncService.listArguments({ funcId })).arguments;
  args.value = argsToEditingArgs(funcArgs);
};

grabArgs(props.funcId);

watch(
  [funcId],
  ([funcId]) => {
    grabArgs(funcId);
  },
  { immediate: true },
);

const addArgument = async () => {
  await FuncService.createArgument({
    funcId: funcId.value,
    name: newArg.value.name,
    kind: newArg.value.kind.value as FuncArgumentKind,
  });
  newArg.value.name = "";
  newArg.value.kind = kindToOption(FuncArgumentKind.String);
  grabArgs(funcId.value);
};
</script>
