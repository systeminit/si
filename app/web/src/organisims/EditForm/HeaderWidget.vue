<template>
  <section>
    <div
      class="flex w-full pt-1 pb-1 mt-2 text-sm text-white"
      :style="propObjectStyle"
    >
      <div v-if="isOpen" class="flex" :style="propObjectStyle">
        <VueFeather type="chevron-down" />
        {{ editField.name }}
      </div>
      <div v-else class="flex" :style="propObjectStyle">
        <VueFeather type="chevron-right" />
        {{ editField.name }}
      </div>
    </div>
  </section>
  <Widgets :edit-fields="widgetEditFields" :indent-level="indentLevel + 1" />
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { EditField, EditFields } from "@/api/sdf/dal/edit_field";
import VueFeather from "vue-feather";
import { defineAsyncComponent, DefineComponent } from "vue";
import type { WidgetsProps } from "./Widgets.vue";

// Eliminate the circular dependency of HeaderWidget -> Widgets -> HeaderWidget
// by using `defineAsyncComponent` in a careful way to preserve the ability for
// typeechecking to work with `tsc` and the `volar` language server used in
// VSCode/NeoVim/Vim.
//
// See:
// https://github.com/johnsoncodehk/volar/issues/644#issuecomment-1012716529
const Widgets = defineAsyncComponent<DefineComponent<WidgetsProps>>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  () => import("./Widgets.vue") as any,
);

const props = defineProps<{
  show: boolean;
  coreEditField: boolean;
  indentLevel: number;
  editField: EditField;
  backgroundColors: number[][];
}>();

// const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

const widgetEditFields = computed<EditFields>(() => {
  if (props.editField.widget.kind == "Header") {
    return props.editField.widget.options.edit_fields;
  } else {
    return [];
  }
});

const isOpen = ref<boolean>(true);

const propObjectStyle = computed<string>(() => {
  // const rgb = props.backgroundColors[1].join(",");
  // let style = `background-color: rgb(${rgb})`;
  // style = `${style} padding-left: ${paddingLeft.value}px;`;
  // return style;
  const backgroundColor = "background-color: rgb(50, 50, 50)";
  const paddingLeft = `padding-left: ${props.indentLevel * 10}px`;
  return `${backgroundColor}; ${paddingLeft};`;
});
</script>

<style scoped>
.property-section-title-bg-color {
  background-color: #292c2d;
}

.section-content {
  @apply overflow-hidden transition duration-150 ease-in-out;
}

.is-closed .section-content {
  @apply overflow-hidden h-0;
}
</style>
