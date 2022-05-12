<template>
  <div>
    <section v-show="show">
      <div
        class="flex w-full pt-1 pb-1 mt-2 text-sm text-white cursor-pointer"
        :style="propObjectStyle"
        @click="toggleHeader"
      >
        <div class="flex" :style="propObjectStyle">
          <VueFeather v-if="openState" type="chevron-down" />
          <VueFeather v-else type="chevron-right" />

          <SiLink
            v-if="props.editField.baggage?.prop_doc_link"
            :uri="props.editField.baggage.prop_doc_link"
            :blank-target="true"
            class="flex flex-row justify-end"
          >
            <span class="flex flex-col content-center justify-center">
              {{ props.editField.name }}
            </span>
            <VueFeather type="help-circle" size="1em" class="m-2" />
          </SiLink>
          <template v-else>
            {{ props.editField.name }}
          </template>
        </div>
      </div>
    </section>
    <Widgets
      :show="showChildren"
      :edit-fields="widgetEditFields"
      :core-edit-fields="props.coreEditField"
      :indent-level="props.indentLevel + 1"
      :tree-open-state="props.treeOpenState"
      :component-identification="props.componentIdentification"
      @toggle-header="bubbleToggleHeader"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { EditField, EditFields } from "@/api/sdf/dal/edit_field";
import VueFeather from "vue-feather";
import { defineAsyncComponent, DefineComponent } from "vue";
import type { WidgetsProps } from "./Widgets.vue";
import { ITreeOpenState } from "@/utils/edit_field_visitor";
import SiLink from "@/atoms/SiLink.vue";
import { ComponentIdentification } from "@/api/sdf/dal/component";

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
  treeOpenState: ITreeOpenState;
  componentIdentification?: ComponentIdentification;
  backgroundColors: number[][];
}>();

const emit = defineEmits<{
  (e: "toggleHeader", fieldId: string): void;
}>();

const widgetEditFields = computed<EditFields>(() => {
  if (props.editField.widget.kind == "Header") {
    return props.editField.widget.options.edit_fields;
  } else {
    return [];
  }
});

const fieldId = computed<string>(() => {
  return props.editField.id;
});

const openState = computed<boolean>(() => {
  const state = props.treeOpenState[fieldId.value];
  if (state === undefined) {
    throw new Error(
      `No open state for fieldId '${fieldId.value}; this is a bug!`,
    );
  }
  return state;
});

const showChildren = computed<boolean>(() => {
  return props.show && openState.value;
});

const toggleHeader = () => {
  emit("toggleHeader", fieldId.value);
};

const bubbleToggleHeader = (fieldId: string) => {
  emit("toggleHeader", fieldId);
};

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
