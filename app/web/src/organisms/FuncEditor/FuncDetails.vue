<template>
  <div v-if="props.funcId > 0" class="text-center h-full overflow-y-auto">
    <SiTabGroup :selected-index="0">
      <template #tabs>
        <SiTabHeader>Properties</SiTabHeader>
      </template>
      <template #panels>
        <TabPanel>
          <SiCollapsible label="Attributes" :default-open="true">
            <SiTextBox
              id="handler"
              v-model="editingFunc.modifiedFunc.handler"
              title="Entrypoint"
              :disabled="editingFunc.origFunc.isBuiltin"
              @blur="updateFunc"
            />
            <SiTextBox
              id="name"
              v-model="editingFunc.modifiedFunc.name"
              title="Name"
              :disabled="editingFunc.origFunc.isBuiltin"
              @blur="updateFunc"
            />
            <SiTextBox
              id="description"
              v-model="editingFunc.modifiedFunc.description"
              placeholder="Provide a brief description of what this qualification validates here..."
              title="Description"
              :text-area="true"
              :disabled="editingFunc.origFunc.isBuiltin"
              @blur="updateFunc"
            />
          </SiCollapsible>
          <SiCollapsible label="Run On" :default-open="true">
            <h1
              class="mb-[0.3125rem] pt-5 text-neutral-700 type-bold-sm dark:text-neutral-50"
            >
              Run on Component:
            </h1>
            <FuncRunOnSelector
              v-model="selectedComponents"
              none-selected-label="select component(s)"
              none-selected-blurb="None selected. Select component(s) above..."
              :options="components"
              :disabled="editingFunc.origFunc.isBuiltin"
            />
            <h1
              class="mb-[0.3125rem] text-neutral-700 type-bold-sm dark:text-neutral-50"
            >
              Run on Schema Variant:
            </h1>
            <FuncRunOnSelector
              v-model="selectedVariants"
              none-selected-label="select schema variant(s)"
              none-selected-blurb="None selected. Select schema variant(s) above..."
              :options="schemaVariants"
              :disabled="editingFunc.origFunc.isBuiltin"
            />
          </SiCollapsible>
        </TabPanel>
      </template>
    </SiTabGroup>
    <div
      class="absolute bottom-0 w-full h-12 text-right p-2 border-t border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800"
    >
      <SiButton
        icon="save"
        kind="save"
        label="Save Qualifications"
        size="lg"
        :disabled="editingFunc.origFunc.isBuiltin"
        @click="saveQualification"
      />
    </div>
  </div>
  <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    Select a function to view its properties.
  </div>
</template>

<script setup lang="ts">
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiButton from "@/atoms/SiButton.vue";
import { Option } from "@/molecules/SelectMenu.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { TabPanel } from "@headlessui/vue";
import { ref, toRef, watch } from "vue";
import { changeFunc, nullEditingFunc } from "./func_state";
import { fromRef, refFrom } from "vuse-rx";
import { map, combineLatestWith } from "rxjs/operators";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { SchematicService } from "@/service/schematic";
import { EditingFunc, funcState$ } from "@/observable/func";
import { ComponentService } from "@/service/component";
import FuncRunOnSelector from "./FuncRunOnSelector.vue";

const props = defineProps<{
  funcId: number;
}>();

const funcId = toRef(props, "funcId", -1);
const funcId$ = fromRef(funcId);

const schemaVariants = refFrom<Option[]>(
  SchematicService.listSchemaVariants().pipe(
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

const editingFunc = refFrom<EditingFunc>(
  funcId$.pipe(
    combineLatestWith(funcState$),
    map(
      ([funcId, funcs]) => funcs.find((f) => f.id == funcId) ?? nullEditingFunc,
    ),
  ),
  nullEditingFunc,
);

const selectedVariants = ref<Option[]>([]);
const selectedComponents = ref<Option[]>([]);

const toOptionValues = (options: Option[], ids: number[]): Option[] =>
  options.filter((opt) =>
    typeof opt.value === "number" ? ids.includes(opt.value) : false,
  );

watch(
  () => editingFunc.value,
  (editingFunc) => {
    selectedVariants.value = toOptionValues(
      schemaVariants.value,
      editingFunc.modifiedFunc.schemaVariants ?? [],
    );

    selectedComponents.value =
      toOptionValues(components.value, editingFunc.modifiedFunc.components) ??
      [];
  },
);

const updateFunc = () => {
  changeFunc({ ...editingFunc.value.modifiedFunc });
};

const saveQualification = () =>
  changeFunc({
    ...editingFunc.value.modifiedFunc,
    components: selectedComponents.value.map(({ value }) => value as number),
    schemaVariants: selectedVariants.value.map(({ value }) => value as number),
  });
</script>
