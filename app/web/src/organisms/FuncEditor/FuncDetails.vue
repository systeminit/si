<template>
  <div
    v-if="props.funcId > 0"
    class="absolute h-full w-full flex flex-col overflow-hidden"
  >
    <SiTabGroup :selected-index="0">
      <template #tabs>
        <SiTabHeader>Properties</SiTabHeader>
      </template>
      <template #panels>
        <TabPanel class="overflow-auto grow">
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
                !editingFunc.isRevertable
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
            v-if="editingFunc.kind === FuncBackendKind.JsQualification"
            label="Run On"
            default-open
          >
            <QualificationDetails
              v-if="associations && associations.type === 'qualification'"
              v-model="associations"
              :components="componentOptions"
              :schema-variants="schemaVariantOptions"
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
              :func-id="props.funcId"
              :disabled="editingFunc.isBuiltin"
            />
          </SiCollapsible>
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
import { ref, toRef, watch, computed } from "vue";
import { refFrom } from "vuse-rx";
import { map, take } from "rxjs/operators";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { Option } from "@/molecules/SelectMenu.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { DiagramService } from "@/service/diagram";
import { EditingFunc } from "@/observable/func";
import { ComponentService } from "@/service/component";
import VButton from "@/molecules/VButton.vue";
import { FuncService, FuncAssociations } from "@/service/func";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import {
  changeFunc,
  funcById,
  funcState,
  insertFunc,
  nullEditingFunc,
} from "./func_state";
import QualificationDetails from "./QualificationDetails.vue";
import FuncArguments from "./FuncArguments.vue";

const props = defineProps<{
  funcId: number;
}>();

const isDevMode = import.meta.env.DEV;

const funcId = toRef(props, "funcId", -1);

const schemaVariantOptions = refFrom<Option[]>(
  DiagramService.listSchemaVariants().pipe(
    map((schemaVariants) =>
      schemaVariants.error
        ? []
        : schemaVariants.map((v) => ({
            label: v.schemaName,
            value: v.id,
          })),
    ),
  ),
  [],
);

const components = refFrom(
  ComponentService.listComponentsIdentification().pipe(
    map((components) => (components.error ? [] : components.list)),
  ),
  [],
);

const componentOptions = computed(() =>
  components.value.map((c) => ({
    label: c.label,
    value: c.value.componentId,
  })),
);

const editingFunc = ref<EditingFunc>(nullEditingFunc);
const associations = ref<FuncAssociations | undefined>(undefined);

watch(
  [funcId, funcState],
  async ([currentFuncId]) => {
    editingFunc.value = funcById(currentFuncId) ?? nullEditingFunc;
    associations.value = editingFunc.value.associations;
  },
  { immediate: true },
);

const updateFunc = () => {
  changeFunc({
    ...editingFunc.value,
    associations: associations.value,
  });
};

const revertFunc = async () => {
  const result = await FuncService.revertFunc({ id: editingFunc.value.id });
  if (result.success) {
    FuncService.getFunc({ id: editingFunc.value.id })
      .pipe(take(1))
      .subscribe(insertFunc);
  }
};

const execFunc = () => {
  FuncService.execFunc({ id: editingFunc.value.id });
};
</script>
