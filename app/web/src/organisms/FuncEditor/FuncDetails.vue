<template>
  <div
    v-if="funcId > 0"
    class="absolute h-full w-full flex flex-col overflow-hidden"
  >
    <SiTabGroup :selected-index="0">
      <template #tabs>
        <SiTabHeader>Properties</SiTabHeader>
        <SiTabHeader v-if="editingFunc.kind === FuncBackendKind.JsAttribute"
          >Bindings
        </SiTabHeader>
      </template>
      <template #panels>
        <TabPanel class="grow">
          <div class="w-full flex p-2 gap-1 border-b dark:border-neutral-600">
            <VButton
              :disabled="!isDevMode && editingFunc.isBuiltin"
              button-rank="primary"
              button-type="success"
              icon="save"
              label="Execute"
              size="md"
              @click="execFunc"
            />

            <VButton
              :disabled="
                (!isDevMode && editingFunc.isBuiltin) ||
                !editingFunc.isRevertible
              "
              button-rank="tertiary"
              button-type="neutral"
              icon="x"
              label="Revert"
              size="sm"
              @click="revertFunc"
            />
          </div>

          <SiCollapsible label="Attributes" default-open>
            <div class="p-3 flex flex-col gap-2">
              <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
                Give this qualification a Name, Entrypoint and brief description
                below.
              </h1>
              <SiTextBox
                id="name"
                v-model="editingFunc.name"
                title="Name"
                required
                placeholder="Type the name of this function here..."
                :disabled="!isDevMode && editingFunc.isBuiltin"
                @blur="updateFunc"
              />
              <SiTextBox
                id="handler"
                v-model="editingFunc.handler"
                title="Entrypoint"
                required
                placeholder="The name of the function that will be executed first..."
                :disabled="!isDevMode && editingFunc.isBuiltin"
                @blur="updateFunc"
              />
              <SiTextBox
                id="description"
                v-model="editingFunc.description"
                placeholder="Provide a brief description of what this qualification validates here..."
                title="Description"
                text-area
                :disabled="!isDevMode && editingFunc.isBuiltin"
                @blur="updateFunc"
              />
            </div>
          </SiCollapsible>
          <SiCollapsible
            v-if="
              editingFunc.kind === FuncBackendKind.JsQualification ||
              editingFunc.kind === FuncBackendKind.JsCodeGeneration ||
              editingFunc.kind === FuncBackendKind.JsConfirmation
            "
            label="Run On"
            default-open
          >
            <QualificationDetails
              v-if="associations && associations.type === 'qualification'"
              v-model="associations"
              :disabled="editingFunc.isBuiltin"
              @change="updateFunc"
            />
            <CodeGenerationDetails
              v-if="associations && associations.type === 'codeGeneration'"
              v-model="associations"
              :disabled="editingFunc.isBuiltin"
              @change="updateFunc"
            />
            <ConfirmationDetails
              v-if="associations && associations.type === 'confirmation'"
              v-model="associations"
              :disabled="editingFunc.isBuiltin"
              @change="updateFunc"
            />
          </SiCollapsible>
          <SiCollapsible
            v-if="editingFunc.kind === FuncBackendKind.JsAttribute"
            label="Arguments"
            default-open
          >
            <FuncArguments
              v-if="associations && associations.type === 'attribute'"
              :func-id="funcId"
              :arguments="associations.arguments"
              :disabled="editingFunc.isBuiltin"
            />
          </SiCollapsible>
        </TabPanel>

        <TabPanel v-if="editingFunc.kind === FuncBackendKind.JsAttribute">
          <AttributeBindings
            v-if="associations && associations.type === 'attribute'"
            :func-id="funcId"
            :associations="associations"
          />
        </TabPanel>
      </template>
    </SiTabGroup>
  </div>
  <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    Select a function to view its properties.
  </div>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { toRef, ref, watch, computed, provide } from "vue";
import { storeToRefs } from "pinia";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { EditingFunc } from "@/observable/func";
import VButton from "@/molecules/VButton.vue";
import { FuncAssociations } from "@/service/func";
import { FuncBackendKind, FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore, nullEditingFunc } from "@/store/funcs.store";
import QualificationDetails from "./QualificationDetails.vue";
import FuncArguments from "./FuncArguments.vue";
import AttributeBindings from "./AttributeBindings.vue";
import CodeGenerationDetails from "./CodeGenerationDetails.vue";
import ConfirmationDetails from "./ConfirmationDetails.vue";

const funcStore = useFuncStore();
const { getFuncById } = storeToRefs(funcStore);

const props = defineProps<{
  funcId?: number;
}>();

const funcId = toRef(props, "funcId", -1);

const isDevMode = import.meta.env.DEV;

const editingFunc = ref<EditingFunc>(nullEditingFunc);
const associations = ref<FuncAssociations | undefined>(undefined);
const funcArgumentsIdMap = computed(() =>
  associations.value?.type === "attribute"
    ? associations.value.arguments.reduce((idMap, arg) => {
        idMap[arg.id] = arg;
        return idMap;
      }, {} as { [key: number]: FuncArgument })
    : {},
);

provide("funcArgumentsIdMap", funcArgumentsIdMap);

watch(
  funcId,
  (newFuncId) => {
    editingFunc.value = getFuncById.value(newFuncId) ?? nullEditingFunc;
    associations.value = editingFunc.value.associations;
  },
  { immediate: true },
);

const updateFunc = () => {
  funcStore.UPDATE_FUNC({
    ...editingFunc.value,
    associations: associations.value,
  });
};

const revertFunc = async () => {
  funcStore.REVERT_FUNC(editingFunc.value.id);
};

const execFunc = () => {
  funcStore.EXEC_FUNC(editingFunc.value.id);
};
</script>
