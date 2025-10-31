<template>
  <div
    :style="{ 'border-left-color': rowData.color }"
    :class="
      clsx(
        'absolute top-0 left-0 w-full flex flex-row items-center px-xs gap-2xs border-l-[3px] cursor-pointer',
        category && themeClasses('bg-neutral-200', 'bg-neutral-700'),
        schema &&
          'hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
        themeClasses(
          'bg-shade-0 hover:outline-action-500',
          'bg-neutral-800 hover:outline-action-300',
        ),
        selected && [
          'add-component-selected-item',
          themeClasses(
            'outline-action-500 bg-action-200',
            'outline-action-300 bg-action-900',
          ),
        ],
      )
    "
  >
    <Icon
      v-if="category"
      size="lg"
      :name="open ? 'chevron--down' : 'chevron--right'"
    />

    <Icon
      v-if="category && category.icon"
      size="md"
      :name="category.icon"
      class="mr-xs"
    />
    {{ rowData.name }}
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { Icon, IconNames, themeClasses } from "@si/vue-lib/design-system";
import { computed } from "vue";
import { UISchemaKey } from "./AddComponentModal.vue";

const props = defineProps<{
  rowData: AddComponentRowData;
  open?: boolean;
  selected: boolean;
  idx: number;
}>();

const category = computed<CategoryRow | undefined>(() => {
  if (props.rowData.type === "category") return props.rowData;
  return undefined;
});

const schema = computed<SchemaRow | undefined>(() => {
  if (props.rowData.type === "schema") return props.rowData;
  return undefined;
});
</script>

<script lang="ts">
interface AddComponentBaseRowData {
  color: string;
  name: string;
}

export type SchemaRow = AddComponentBaseRowData & {
  type: "schema";
  key: UISchemaKey;
};
export type CategoryRow = AddComponentBaseRowData & {
  type: "category";
  icon?: IconNames;
};

export type AddComponentRowData = SchemaRow | CategoryRow;
</script>
