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
              :disabled="!isDevMode && editingFunc.origFunc.isBuiltin"
              button-rank="primary"
              button-type="success"
              icon="save"
              label="Save"
              size="md"
              @click="saveQualification"
            />

            <VButton
              :disabled="!isDevMode && editingFunc.origFunc.isBuiltin"
              button-rank="tertiary"
              button-type="neutral"
              icon="x"
              label="Cancel"
              size="sm"
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
                v-model="editingFunc.modifiedFunc.name"
                title="Name"
                required
                placeholder="Type the name of this function here..."
                :disabled="!isDevMode && editingFunc.origFunc.isBuiltin"
                @blur="updateFunc"
              />
              <SiTextBox
                id="handler"
                v-model="editingFunc.modifiedFunc.handler"
                title="Entrypoint"
                required
                placeholder="The name of the function that will be executed first..."
                :disabled="!isDevMode && editingFunc.origFunc.isBuiltin"
                @blur="updateFunc"
              />
              <SiTextBox
                id="description"
                v-model="editingFunc.modifiedFunc.description"
                placeholder="Provide a brief description of what this qualification validates here..."
                title="Description"
                text-area
                :disabled="!isDevMode && editingFunc.origFunc.isBuiltin"
                @blur="updateFunc"
              />
            </div>
          </SiCollapsible>
          <SiCollapsible label="Run On" default-open>
            <div class="p-3 flex flex-col gap-2">
              <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
                Run this qualification on the selected components and component
                types below.
              </h1>
              <h2
                class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50"
              >
                Run on Component:
              </h2>
              <FuncRunOnSelector
                v-model="selectedComponents"
                thing-label="components"
                :options="components"
                :disabled="editingFunc.origFunc.isBuiltin"
              />
              <h2
                class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50"
              >
                Run on Schema Variant:
              </h2>
              <FuncRunOnSelector
                v-model="selectedVariants"
                thing-label="schema variants"
                :options="schemaVariants"
                :disabled="editingFunc.origFunc.isBuiltin"
              />
            </div>
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
import { ref, toRef, watch } from "vue";
import { refFrom } from "vuse-rx";
import { map } from "rxjs/operators";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { Option } from "@/molecules/SelectMenu.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { DiagramService } from "@/service/diagram";
import { EditingFunc } from "@/observable/func";
import { ComponentService } from "@/service/component";
import VButton from "@/molecules/VButton.vue";
import { changeFunc, funcById, funcState, nullEditingFunc } from "./func_state";
import FuncRunOnSelector from "./FuncRunOnSelector.vue";

const props = defineProps<{
  funcId: number;
}>();

const isDevMode = import.meta.env.DEV;

const funcId = toRef(props, "funcId", -1);

const schemaVariants = refFrom<Option[]>(
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

const components = refFrom<Option[]>(
  ComponentService.listComponentsIdentification().pipe(
    map((components) =>
      components.error
        ? []
        : components.list.map((c) => ({
            label: c.label,
            value: c.value.componentId,
          })),
    ),
  ),
  [],
);

const editingFunc = ref<EditingFunc>(nullEditingFunc);
const selectedVariants = ref<Option[]>([]);
const selectedComponents = ref<Option[]>([]);

const toOptionValues = (options: Option[], ids: number[]): Option[] =>
  options.filter((opt) =>
    typeof opt.value === "number" ? ids.includes(opt.value) : false,
  );

watch([funcId, funcState], async ([currentFuncId]) => {
  editingFunc.value = funcById(currentFuncId) ?? nullEditingFunc;

  selectedVariants.value = toOptionValues(
    schemaVariants.value,
    editingFunc.value.modifiedFunc.schemaVariants ?? [],
  );

  selectedComponents.value =
    toOptionValues(
      components.value,
      editingFunc.value.modifiedFunc.components,
    ) ?? [];
});

const updateFunc = () => {
  changeFunc({ ...editingFunc.value.modifiedFunc });
};

const saveQualification = () => {
  changeFunc({
    ...editingFunc.value.modifiedFunc,
    components: selectedComponents.value.map(({ value }) => value as number),
    schemaVariants: selectedVariants.value.map(({ value }) => value as number),
  });
};
</script>
