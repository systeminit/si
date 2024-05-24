<template>
  <div class="p-3 flex flex-col gap-xs">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Add the names of the arguments to this function and their types.
    </h1>
    <Inline alignY="center">
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
        noLabel
        type="dropdown"
        :options="kindOptions"
        @change="
          updateArgument(
            newArg.id,
            newArg.name,
            newArg.kind,
            newArg.elementKind,
          )
        "
      />
      <VButton
        label="Add"
        tone="success"
        :disabled="disabled"
        @click="createArgument"
      />
    </Inline>
    <ul>
      <Inline v-for="arg in funcArguments" :key="arg.id" alignY="center">
        <VormInput
          :id="`arg-name-${arg.id}`"
          v-model="arg.name"
          type="text"
          noLabel
          placeholder="Argument name"
          :disabled="disabled"
          @blur="updateArgument(arg.id, arg.name, arg.kind, arg.elementKind)"
        />
        <VormInput
          v-model="arg.kind"
          noLabel
          type="dropdown"
          :options="kindOptions"
          @change="updateArgument(arg.id, arg.name, arg.kind, arg.elementKind)"
        />
        <VButton
          label="Delete"
          tone="destructive"
          :disabled="disabled"
          @click="deleteArgument(arg.id)"
        />
      </Inline>
    </ul>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref } from "vue";
import { Inline, VButton, VormInput } from "@si/vue-lib/design-system";
import { FuncArgument, FuncArgumentKind } from "@/api/sdf/dal/func";
import { Option } from "@/components/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";

const props = defineProps<{
  funcId: string;
  disabled?: boolean;
}>();

const funcsStore = useFuncStore();

const generateKindOptions = () => {
  const options: Option[] = [];
  for (const kind in FuncArgumentKind) {
    options.push({ label: kind, value: kind });
  }
  return options;
};

const kindOptions = generateKindOptions();
// we haven't implemented element kinds yet
// const elementKindOptions = [kindToOption()].concat(generateKindOptions());

const newArg = ref<FuncArgument>({
  id: "",
  name: "",
  kind: FuncArgumentKind.String,
});

const resetNewArg = () => {
  newArg.value.name = "";
  newArg.value.kind = FuncArgumentKind.String;
};

const createArgument = async () => {
  await funcsStore.CREATE_FUNC_ARGUMENT(
    props.funcId,
    newArg.value.name,
    newArg.value.kind,
    newArg.value.elementKind,
  );

  resetNewArg();
};

const updateArgument = async (
  funcArgumentId: string,
  name: string,
  kind: FuncArgumentKind,
  elementKind?: FuncArgumentKind,
) => {
  await funcsStore.UPDATE_FUNC_ARGUMENT(
    props.funcId,
    funcArgumentId,
    name,
    kind,
    elementKind,
  );
};

const funcArguments = computed(
  () => funcsStore.funcArgumentsByFuncId[props.funcId],
);

const deleteArgument = async (funcArgumentId: string) => {
  await funcsStore.DELETE_FUNC_ARGUMENT(props.funcId, funcArgumentId);
};

onMounted(() => {
  funcsStore.FETCH_FUNC_ARGUMENT_LIST(props.funcId);
});
</script>
