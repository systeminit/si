<template>
  <div class="p-xs flex flex-col gap-xs">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Add the names of the arguments to this function and their types.
    </h1>
    <div class="flex flex-row items-center gap-2xs">
      <VormInput
        id="newArg"
        v-model="newArg.name"
        type="text"
        noLabel
        placeholder="New argument name..."
        :disabled="disabled"
      />
      <VormInput
        v-model="newArg.kind"
        class="flex-none"
        noLabel
        type="dropdown"
        :disabled="disabled"
        :options="kindOptions"
      />
      <VButton icon="plus" size="sm" tone="success" :disabled="disabled" @click="createArgument" />
    </div>
    <ul>
      <div v-for="arg in funcArguments" :key="arg.id" class="flex flex-row items-center gap-2xs">
        <VormInput
          v-model="arg.name"
          type="text"
          noLabel
          placeholder="Argument name"
          disabled
          @blur="updateArgument(arg)"
        />
        <VormInput
          v-model="arg.kind"
          class="flex-none"
          noLabel
          type="dropdown"
          :options="kindOptions"
          disabled
          @change="updateArgument(arg)"
        />
        <VButton icon="trash" size="sm" tone="destructive" @click="deleteArgument(arg.id)" />
      </div>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { VButton, VormInput } from "@si/vue-lib/design-system";
import { FuncArgument, FuncArgumentKind } from "@/api/sdf/dal/func";
import { Option } from "@/components/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { nilId } from "@/utils/nilId";

const props = defineProps<{
  funcId: string;
  disabled?: boolean;
}>();

const funcsStore = useFuncStore();

const generateKindOptions = () => {
  const options: Option[] = [];
  for (const kind in FuncArgumentKind) {
    options.push({ label: kind, value: kind.toLowerCase() });
  }
  return options;
};

const kindOptions = generateKindOptions();

const newArg = ref<FuncArgument>({
  id: nilId(),
  name: "",
  kind: FuncArgumentKind.String,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
});

const resetNewArg = () => {
  newArg.value.name = "";
  newArg.value.kind = FuncArgumentKind.String;
};

const createArgument = async () => {
  await funcsStore.CREATE_FUNC_ARGUMENT(props.funcId, newArg.value);

  resetNewArg();
};

const updateArgument = async (arg: FuncArgument) => {
  await funcsStore.UPDATE_FUNC_ARGUMENT(props.funcId, arg);
};

const funcArguments = computed(() => funcsStore.funcsById[props.funcId]?.arguments);

const deleteArgument = async (funcArgumentId: string) => {
  await funcsStore.DELETE_FUNC_ARGUMENT(props.funcId, funcArgumentId);
};
</script>
