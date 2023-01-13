<template>
  <div
    v-if="funcReqStatus.isPending"
    class="w-full flex flex-col items-center gap-4 p-xl"
  >
    <Icon name="loader" size="2xl" />
  </div>
  <div
    v-else-if="selectedFuncId !== nilId()"
    class="absolute h-full w-full flex flex-col overflow-hidden"
  >
    <SiTabGroup>
      <template #tabs>
        <SiTabHeader>Properties</SiTabHeader>
        <SiTabHeader v-if="editingFunc.variant === FuncVariant.Attribute"
          >Bindings
        </SiTabHeader>
      </template>
      <template #panels>
        <TabPanel class="grow">
          <div class="w-full flex p-2 gap-1 border-b dark:border-neutral-600">
            <VButton2
              class="--tone-success"
              :disabled="!isDevMode && editingFunc.isBuiltin"
              icon="save"
              size="md"
              loading-text="Executing"
              label="Execute"
              :request-status="execFuncReqStatus"
              @click="execFunc"
            />

            <VButton2
              class="--tone-neutral"
              :disabled="
                (!isDevMode && editingFunc.isBuiltin) ||
                !editingFunc.isRevertible
              "
              icon="x"
              size="md"
              loading-text="Reverting"
              label="Revert"
              :request-status="revertFuncReqStatus"
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
                placeholder="The name of the function that will be executed..."
                :disabled="!isDevMode && editingFunc.isBuiltin"
                @blur="updateFunc"
              />
              <SiTextBox
                id="description"
                v-model="editingFunc.description"
                placeholder="Provide a brief description of this function here..."
                title="Description"
                text-area
                :disabled="!isDevMode && editingFunc.isBuiltin"
                @blur="updateFunc"
              />
            </div>
          </SiCollapsible>
          <SiCollapsible
            v-if="
              editingFunc.variant === FuncVariant.CodeGeneration ||
              editingFunc.variant === FuncVariant.Confirmation ||
              editingFunc.variant === FuncVariant.Qualification ||
              editingFunc.variant === FuncVariant.Validation
            "
            label="Run On"
            default-open
          >
            <QualificationDetails
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'qualification'
              "
              v-model="editingFunc.associations"
              :disabled="editingFunc.isBuiltin"
              @change="updateFunc"
            />
            <CodeGenerationDetails
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'codeGeneration'
              "
              v-model="editingFunc.associations"
              :disabled="editingFunc.isBuiltin"
              @change="updateFunc"
            />
            <ConfirmationDetails
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'confirmation'
              "
              v-model="editingFunc.associations"
              :disabled="editingFunc.isBuiltin"
              @change="updateFunc"
            />
            <ValidationDetails
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'validation'
              "
              v-model="editingFunc.associations"
              :disabled="editingFunc.isBuiltin"
              @change="updateFunc"
            />
          </SiCollapsible>
          <SiCollapsible
            v-if="editingFunc.variant === FuncVariant.Attribute"
            label="Arguments"
            default-open
          >
            <FuncArguments
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'attribute'
              "
              :func-id="selectedFuncId"
              :arguments="editingFunc.associations.arguments"
              :disabled="editingFunc.isBuiltin"
            />
          </SiCollapsible>
        </TabPanel>

        <TabPanel v-if="editingFunc.variant === FuncVariant.Attribute">
          <AttributeBindings
            v-if="
              editingFunc.associations &&
              editingFunc.associations.type === 'attribute'
            "
            :func-id="selectedFuncId"
            :associations="editingFunc.associations"
          />
        </TabPanel>
      </template>
    </SiTabGroup>
  </div>
  <div
    v-else
    class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
  >
    <template v-if="funcId">Function "{{ funcId }}" does not exist!</template>
    <template v-else>Select a function to view its properties.</template>
  </div>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { computed, provide } from "vue";
import { storeToRefs } from "pinia";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import { FuncVariant, FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore, nullEditingFunc } from "@/store/func/funcs.store";
import Icon from "@/ui-lib/icons/Icon.vue";
import QualificationDetails from "./QualificationDetails.vue";
import FuncArguments from "./FuncArguments.vue";
import AttributeBindings from "./AttributeBindings.vue";
import CodeGenerationDetails from "./CodeGenerationDetails.vue";
import ConfirmationDetails from "./ConfirmationDetails.vue";
import ValidationDetails from "./ValidationDetails.vue";

defineProps({
  funcId: { type: String },
});

function nilId(): string {
  return "00000000000000000000000000";
}

const funcStore = useFuncStore();
const funcReqStatus = funcStore.getRequestStatus("FETCH_FUNC");
const { getFuncById, selectedFuncId } = storeToRefs(funcStore);

const isDevMode = import.meta.env.DEV;
const funcArgumentsIdMap = computed(() =>
  editingFunc?.value?.associations?.type === "attribute"
    ? editingFunc?.value?.associations.arguments.reduce((idMap, arg) => {
        idMap[arg.id] = arg;
        return idMap;
      }, {} as { [key: string]: FuncArgument })
    : {},
);

provide("funcArgumentsIdMap", funcArgumentsIdMap);

const editingFunc = computed(
  () => getFuncById.value(selectedFuncId.value) ?? nullEditingFunc,
);

const updateFunc = () => {
  funcStore.updateFuncAssociations(
    editingFunc.value.id,
    editingFunc.value.associations,
  );
};

const revertFuncReqStatus = funcStore.getRequestStatus("REVERT_FUNC");
const revertFunc = async () => {
  funcStore.REVERT_FUNC(editingFunc.value.id);
};

const execFuncReqStatus = funcStore.getRequestStatus("EXEC_FUNC");
const execFunc = () => {
  funcStore.EXEC_FUNC(editingFunc.value.id);
};
</script>
