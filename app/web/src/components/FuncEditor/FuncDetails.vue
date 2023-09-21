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
  >
    Loading function "{{ selectedFuncSummary?.name }}"
  </LoadingMessage>
  <div
    v-else-if="selectedFuncId && editingFunc"
    :class="
      clsx(
        singleModelScreen ? 'relative' : 'absolute',
        'h-full w-full flex flex-col overflow-hidden',
      )
    "
  >
    <TabGroup rememberSelectedTabKey="func_details">
      <TabGroupItem label="Properties" slug="properties">
        <ScrollArea>
          <Stack class="p-2 border-b dark:border-neutral-600" spacing="xs">
            <div class="flex gap-1">
              <VButton
                class="--tone-success"
                icon="save"
                size="md"
                loadingText="Executing..."
                label="Execute"
                :requestStatus="execFuncReqStatus"
                successText="Finished"
                @click="execFunc"
              />

              <VButton
                class="--tone-neutral"
                :disabled="!isRevertible"
                icon="x"
                size="md"
                loadingText="Reverting..."
                label="Revert"
                :requestStatus="revertFuncReqStatus"
                successText="Finished"
                @click="revertFunc"
              />

              <VButton
                v-if="schemaVariantId"
                :loading="isDetaching"
                tone="destructive"
                icon="x"
                label="Detach"
                size="md"
                loadingText="Detaching..."
                @click="detachFunc"
              />
            </div>

            <ErrorMessage
              v-if="execFuncReqStatus.isError"
              :requestStatus="execFuncReqStatus"
            />
            <ErrorMessage
              v-if="editingFunc?.associations?.type === 'action'"
              icon="alert-triangle"
              tone="warning"
              >Executing will run on all attached components, so may have
              effects on your real world resources!</ErrorMessage
            >
            <ErrorMessage
              v-if="isConnectedToOtherAssetTypes"
              icon="alert-triangle"
              tone="warning"
            >
              This function is connected to other
              {{
                (editingFunc?.associations &&
                  editingFunc.associations?.type === "validation") ||
                (editingFunc?.associations &&
                  editingFunc?.associations?.type === "attribute")
                  ? "attributes"
                  : "assets"
              }}.
            </ErrorMessage>
          </Stack>

          <!-- <Collapsible label="Logs">
            {{ lastExecutionLog }}
            <VButton @click="getLastExecution">Load</VButton>
          </Collapsible> -->
          <Collapsible label="Attributes" defaultOpen>
            <Stack class="p-3">
              <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
                Give this function a Name, Entrypoint and brief description
                below.
              </h1>
              <VormInput
                v-model="editingFunc.name"
                label="Name"
                required
                placeholder="Type the name of this function here..."
                @blur="updateFunc"
              />
              <VormInput
                v-model="editingFunc.displayName"
                label="Display Name"
                required
                placeholder="Type the display name of this function here..."
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
            </Stack>
          </Collapsible>
          <ActionDetails
            v-if="
              editingFunc.associations &&
              editingFunc.associations.type === 'action'
            "
            ref="detachRef"
            v-model="editingFunc.associations"
            :requestStatus="updateFuncReqStatus"
            :schemaVariantId="schemaVariantId"
            @change="updateFunc"
          />
          <CodeGenerationDetails
            v-if="
              editingFunc.associations &&
              editingFunc.associations.type === 'codeGeneration'
            "
            v-model="editingFunc.associations"
            :schemaVariantId="schemaVariantId"
            @change="updateFunc"
          />
          <QualificationDetails
            v-if="
              editingFunc.associations &&
              editingFunc.associations.type === 'qualification'
            "
            ref="detachRef"
            v-model="editingFunc.associations"
            :schemaVariantId="schemaVariantId"
            @change="updateFunc"
          />
          <ValidationDetails
            v-if="
              editingFunc.associations &&
              editingFunc.associations.type === 'validation'
            "
            ref="detachRef"
            v-model="editingFunc.associations"
            :schemaVariantId="schemaVariantId"
            @change="updateFunc"
          />

          <Collapsible
            v-if="editingFunc.variant === FuncVariant.Attribute"
            label="Arguments"
            defaultOpen
          >
            <FuncArguments
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'attribute'
              "
              v-model="editingFunc.associations"
              @change="updateFunc"
            />
          </Collapsible>

          <Collapsible
            v-if="
              editingFunc.variant === FuncVariant.Attribute && schemaVariantId
            "
            label="Binding"
            defaultOpen
          >
            <AttributeBindings
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'attribute'
              "
              ref="detachRef"
              v-model="editingFunc.associations"
              :schemaVariantId="schemaVariantId"
              @change="updateFunc"
            />
          </Collapsible>
        </ScrollArea>
      </TabGroupItem>

      <TabGroupItem
        v-if="editingFunc.variant === FuncVariant.Attribute && !schemaVariantId"
        label="Bindings"
        slug="bindings"
      >
        <AttributeBindings
          v-if="
            editingFunc.associations &&
            editingFunc.associations.type === 'attribute'
          "
          ref="detachRef"
          v-model="editingFunc.associations"
          :schemaVariantId="schemaVariantId"
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
  Collapsible,
  VButton,
  TabGroup,
  TabGroupItem,
  LoadingMessage,
  VormInput,
  Stack,
  ErrorMessage,
  ScrollArea,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { FuncVariant, FuncArgument } from "@/api/sdf/dal/func";
import { useFuncStore, FuncId } from "@/store/func/funcs.store";
import FuncArguments from "./FuncArguments.vue";
import ActionDetails from "./ActionDetails.vue";
import AttributeBindings from "./AttributeBindings.vue";
import CodeGenerationDetails from "./CodeGenerationDetails.vue";
import ValidationDetails from "./ValidationDetails.vue";
import QualificationDetails from "./QualificationDetails.vue";

const props = defineProps<{
  funcId?: FuncId;
  schemaVariantId?: string;
  singleModelScreen?: boolean;
}>();

const funcStore = useFuncStore();

const emit = defineEmits<{ (e: "detached"): void }>();

type DetachType =
  | InstanceType<typeof ActionDetails>
  | InstanceType<typeof AttributeBindings>
  | InstanceType<typeof CodeGenerationDetails>
  | InstanceType<typeof ValidationDetails>
  | InstanceType<typeof QualificationDetails>;

const detachRef = ref<DetachType>();
const funcId = computed(() => props.funcId);

const loadFuncDetailsReqStatus = funcStore.getRequestStatus(
  "FETCH_FUNC_DETAILS",
  funcId,
);
const updateFuncReqStatus = funcStore.getRequestStatus("UPDATE_FUNC", funcId);
const { selectedFuncId, selectedFuncSummary } = storeToRefs(funcStore);

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
const editingFunc = ref(_.cloneDeep(storeFuncDetails.value));

function resetEditingFunc() {
  editingFunc.value = _.cloneDeep(storeFuncDetails.value);
}

// when the func details finish loading, we copy into our local draft
watch([loadFuncDetailsReqStatus, updateFuncReqStatus], () => {
  resetEditingFunc();
});

const isRevertible = computed(() =>
  funcId.value ? funcStore.funcDetailsById[funcId.value]?.isRevertible : false,
);

const updateFunc = () => {
  if (
    !editingFunc.value ||
    _.isEqual(editingFunc.value, storeFuncDetails.value)
  )
    return;
  funcStore.UPDATE_FUNC(editingFunc.value);
};

const revertFuncReqStatus = funcStore.getRequestStatus("REVERT_FUNC");
const revertFunc = async () => {
  if (!funcId.value) return;
  await funcStore.REVERT_FUNC(funcId.value);
  await funcStore.FETCH_FUNC_DETAILS(funcId.value);
  resetEditingFunc();
};

const isConnectedToOtherAssetTypes = computed(() => {
  if (editingFunc?.value && editingFunc?.value?.associations) {
    const associations = editingFunc.value.associations;
    switch (associations.type) {
      case "codeGeneration":
      case "qualification":
        return (
          associations.schemaVariantIds.length > 1 ||
          associations.componentIds.length > 1
        );
      case "action":
        return associations.schemaVariantIds.length > 1;
      case "validation":
        return associations.prototypes.length > 1;
      case "attribute":
        return associations.prototypes.length > 1;
      default:
        return false;
    }
  }
  return false;
});

const execFuncReqStatus = funcStore.getRequestStatus(
  "SAVE_AND_EXEC_FUNC",
  funcId,
);
const execFunc = () => {
  if (!funcId.value) return;
  funcStore.SAVE_AND_EXEC_FUNC(funcId.value);
};

const isDetaching = ref(false);
const detachFunc = async () => {
  if (detachRef.value && "detachFunc" in detachRef.value) {
    const associations = detachRef.value.detachFunc();
    if (associations && editingFunc.value) {
      isDetaching.value = true;
      await funcStore.UPDATE_FUNC({
        ...editingFunc.value,
        associations,
      });
      emit("detached");
      isDetaching.value = false;
    }
  }
};

// const getExecutionReqStatus = funcStore.getRequestStatus(
//   "GET_FUNC_LAST_EXECUTION",
//   funcId,
// );
// function getLastExecution() {
//   if (!funcId.value) return;
//   funcStore.GET_FUNC_LAST_EXECUTION(funcId.value);
// }
// const lastExecutionLog = computed(
//   () => funcStore.lastFuncExecutionLogByFuncId[funcId?.value || ""],
// );
</script>
