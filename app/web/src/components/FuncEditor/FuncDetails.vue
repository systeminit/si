<template>
  <div
    v-if="!funcId"
    class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
  >
    Select a function to view its properties.
  </div>
  <LoadingMessage
    v-else-if="
      (loadFuncDetailsReqStatus.isPending && !storeFuncDetails) ||
      !loadFuncDetailsReqStatus.isRequested
    "
    no-message
  />
  <div
    v-else-if="selectedFuncId && editingFunc"
    class="absolute h-full w-full flex flex-col overflow-hidden"
  >
    <TabGroup remember-selected-tab-key="func_details">
      <TabGroupItem label="Properties" slug="properties">
        <template #top>
          <Stack>
            <div class="w-full flex p-2 gap-1 border-b dark:border-neutral-600">
              <VButton2
                class="--tone-success"
                icon="save"
                size="md"
                loading-text="Executing"
                label="Execute"
                :request-status="execFuncReqStatus"
                success-text="Finished"
                @click="execFunc"
              />

              <VButton2
                class="--tone-neutral"
                :disabled="!isRevertible"
                icon="x"
                size="md"
                loading-text="Reverting"
                label="Revert"
                :request-status="revertFuncReqStatus"
                success-text="Finished"
                @click="revertFunc"
              />
            </div>
            <div class="p-2">
              <ErrorMessage
                v-if="execFuncReqStatus.isError"
                :request-status="execFuncReqStatus"
              />
            </div>
          </Stack>
        </template>

        <SiCollapsible label="Attributes" default-open>
          <div class="p-3 flex flex-col gap-2">
            <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
              Give this function a Name, Entrypoint and brief description below.
            </h1>
            <VormInput
              v-model="editingFunc.name"
              label="Name"
              required
              placeholder="Type the name of this function here..."
              @blur="updateFunc"
            />
            <VormInput
              v-model="editingFunc.handler"
              label="Entrypoint"
              required
              placeholder="The name of the function that will be executed..."
              @blur="updateFunc"
            />
            <VormInput
              v-model="editingFunc.description"
              type="textarea"
              placeholder="Provide a brief description of this function here..."
              label="Description"
              @blur="updateFunc"
            />
          </div>
        </SiCollapsible>
        <QualificationDetails
          v-if="
            editingFunc.associations &&
            editingFunc.associations.type === 'qualification'
          "
          v-model="editingFunc.associations"
          @change="updateFunc"
        />
        <CodeGenerationDetails
          v-if="
            editingFunc.associations &&
            editingFunc.associations.type === 'codeGeneration'
          "
          v-model="editingFunc.associations"
          @change="updateFunc"
        />
        <ConfirmationDetails
          v-if="
            editingFunc.associations &&
            editingFunc.associations.type === 'confirmation'
          "
          v-model="editingFunc.associations"
          @change="updateFunc"
        />
        <ValidationDetails
          v-if="
            editingFunc.associations &&
            editingFunc.associations.type === 'validation'
          "
          v-model="editingFunc.associations"
          @change="updateFunc"
        />

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
            v-model="editingFunc.associations"
            @change="updateFunc"
          />
        </SiCollapsible>
      </TabGroupItem>

      <TabGroupItem
        v-if="editingFunc.variant === FuncVariant.Attribute"
        label="Bindings"
        slug="bindings"
      >
        <AttributeBindings
          v-if="
            editingFunc.associations &&
            editingFunc.associations.type === 'attribute'
          "
          v-model="editingFunc.associations"
          @change="updateFunc"
        />
      </TabGroupItem>
    </TabGroup>
  </div>
  <div
    v-else
    class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
  >
    Function "{{ funcId }}" does not exist!
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, provide, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import {
  VButton2,
  TabGroup,
  TabGroupItem,
  LoadingMessage,
  VormInput,
  Stack,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import SiCollapsible from "@/components/SiCollapsible.vue";
import { FuncVariant, FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import FuncArguments from "./FuncArguments.vue";
import AttributeBindings from "./AttributeBindings.vue";
import CodeGenerationDetails from "./CodeGenerationDetails.vue";
import ConfirmationDetails from "./ConfirmationDetails.vue";
import ValidationDetails from "./ValidationDetails.vue";
import QualificationDetails from "./QualificationDetails.vue";

const funcStore = useFuncStore();

// NOT REACTIVE - parent has a key so this component rerenders if this changes
const funcId = funcStore.urlSelectedFuncId;

const loadFuncDetailsReqStatus = funcStore.getRequestStatus(
  "FETCH_FUNC_DETAILS",
  funcId,
);
const { selectedFuncId } = storeToRefs(funcStore);

const funcArgumentsIdMap = computed(() =>
  editingFunc?.value?.associations?.type === "attribute"
    ? editingFunc?.value?.associations.arguments.reduce((idMap, arg) => {
        idMap[arg.id] = arg;
        return idMap;
      }, {} as { [key: string]: FuncArgument })
    : {},
);

provide("funcArgumentsIdMap", funcArgumentsIdMap);

const storeFuncDetails = computed(() => funcStore.selectedFuncDetails);
const editingFunc = ref(storeFuncDetails.value);

function resetEditingFunc() {
  editingFunc.value = _.cloneDeep(storeFuncDetails.value);
}

// when the func details finish loading, we copy into our local draft
watch(loadFuncDetailsReqStatus, () => {
  if (loadFuncDetailsReqStatus.value.isSuccess) {
    resetEditingFunc();
  }
});

const isRevertible = computed(() =>
  funcId ? funcStore.funcDetailsById[funcId]?.isRevertible : false,
);

const updateFunc = () => {
  if (!funcId || !editingFunc.value) return;
  funcStore.updateFuncMetadata(editingFunc.value);
};

const revertFuncReqStatus = funcStore.getRequestStatus("REVERT_FUNC");
const revertFunc = async () => {
  if (!funcId) return;
  await funcStore.REVERT_FUNC(funcId);
  resetEditingFunc();
};

const execFuncReqStatus = funcStore.getRequestStatus("SAVE_AND_EXEC_FUNC");
const execFunc = () => {
  if (!funcId) return;
  funcStore.SAVE_AND_EXEC_FUNC(funcId);
};
</script>
