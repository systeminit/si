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
            <!--            <SiTextBox
              id="description"
              placeholder="Provide a brief description of what this qualification validates here..."
              title="Description"
              :text-area="true"
              :disabled="editingFunc.origFunc.isBuiltin"
              />-->
          </SiCollapsible>
          <SiCollapsible label="Run On" :default-open="true">
            <h1
              class="mb-[0.3125rem] text-neutral-700 type-bold-sm dark:text-neutral-50"
            >
              Run on Schema Variant:
            </h1>
            <SelectMenu
              v-model="selectedVariant"
              class="flex-grow"
              :options="variantOptions"
              :disabled="editingFunc.origFunc.isBuiltin"
              @change="selectVariant"
            />
          </SiCollapsible>
        </TabPanel>
      </template>
    </SiTabGroup>
    <div
      class="absolute bottom-0 w-full h-12 text-right p-2 border-t border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800"
    >
      <SiButton icon="save" kind="save" label="Save Qualifications" size="lg" />
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
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { TabPanel } from "@headlessui/vue";
import { ref, toRef, computed, watch } from "vue";
import { changeFunc, nullEditingFunc } from "./func_state";
import { fromRef, refFrom } from "vuse-rx";
import { map, combineLatestWith } from "rxjs/operators";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { ListSchemaVariantsResponse } from "@/service/schematic/list_schema_variants";
import { SchematicService } from "@/service/schematic";
import { EditingFunc, funcState$ } from "@/observable/func";

const props = defineProps<{
  funcId: number;
}>();

const funcId = toRef(props, "funcId", -1);
const funcId$ = fromRef(funcId);

const schemaVariants = refFrom<ListSchemaVariantsResponse>(
  SchematicService.listSchemaVariants().pipe(
    map((schemaVariants) =>
      schemaVariants.error
        ? []
        : (schemaVariants as ListSchemaVariantsResponse),
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

const noVariantSelected: Option = { label: "- none -", value: -1 };

const variantOptions = computed(() =>
  [noVariantSelected].concat(
    schemaVariants.value.map((v) => ({
      label: v.schemaName,
      value: v.id,
    })),
  ),
);

const findSelectedVariant = (editingFunc: EditingFunc) =>
  variantOptions.value.find(
    (opt) => opt.value === (editingFunc.modifiedFunc.schemaVariants[0] ?? -1),
  ) ?? noVariantSelected;

const selectedVariant = ref<Option>(noVariantSelected);

watch(
  () => editingFunc.value,
  (editingFunc) => {
    selectedVariant.value = findSelectedVariant(editingFunc);
  },
);

const selectVariant = (option: Option) => {
  changeFunc({
    ...editingFunc.value.modifiedFunc,
    schemaVariants: [typeof option.value === "string" ? -1 : option.value],
  });
};

const updateFunc = () => {
  changeFunc({ ...editingFunc.value.modifiedFunc });
};
</script>
