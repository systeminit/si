<template>
  <div
    :style="{ 'border-left-color': rowData.color }"
    :class="
      clsx(
        'absolute top-0 left-0 w-full flex flex-row items-center px-xs gap-2xs border-l-[3px] cursor-pointer',
        category && themeClasses('bg-neutral-200', 'bg-neutral-700'),
        category &&
          themeClasses('hover:text-action-500', 'hover:text-action-300'),
        schema &&
          'hover:outline hover:z-10 hover:-outline-offset-1 hover:outline-1',
        selected
          ? [
              'add-component-selected-item',
              createFailed
                ? themeClasses(
                    'outline-destructive-500 bg-destructive-200',
                    'outline-destructive-400 bg-destructive-900',
                  )
                : themeClasses(
                    'outline-action-500 bg-action-200',
                    'outline-action-300 bg-action-900',
                  ),
            ]
          : themeClasses(
              'bg-shade-0 hover:outline-action-500',
              'bg-neutral-800 hover:outline-action-300',
            ),
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
    <div class="flex flex-row items-center gap-xs flex-1 min-w-0">
      <TruncateWithTooltip>
        {{ rowData.name }}
      </TruncateWithTooltip>
      <EditingPill
        v-if="'editing' in rowData && rowData.editing"
        :color="rowData.color"
      />
    </div>
    <div class="ml-auto flex flex-none max-w-full truncate">
      <Icon v-if="submitted" name="loader" size="sm" />
      <div
        v-else-if="selected"
        :class="
          clsx('text-xs', themeClasses('text-neutral-900', 'text-neutral-200'))
        "
      >
        <div v-if="createFailed" class="flex flex-row items-center">
          <Icon name="x" class="text-destructive-500" /> Component creation
          failed
        </div>
        <template v-else>
          <TextPill tighter variant="key2">Enter</TextPill> to add
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import {
  Icon,
  IconNames,
  themeClasses,
  TruncateWithTooltip,
  TextPill,
} from "@si/vue-lib/design-system";
import { computed } from "vue";
import EditingPill from "@/components/EditingPill.vue";
import { UISchemaKey } from "./AddComponentModal.vue";

const props = defineProps<{
  rowData: AddComponentRowData;
  open?: boolean;
  selected: boolean;
  submitted: boolean;
  createFailed: boolean;
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
  editing: boolean;
  key: UISchemaKey;
};
export type CategoryRow = AddComponentBaseRowData & {
  type: "category";
  icon?: IconNames;
};

export type AddComponentRowData = SchemaRow | CategoryRow;
</script>
