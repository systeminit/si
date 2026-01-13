<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center border focus-within:z-10',
        themeClasses(
          'text-black bg-white border-neutral-400 focus-within:border-action-500',
          'text-white bg-black border-neutral-600 focus-within:border-action-300',
        ),
      )
    "
  >
    <component
      :is="isTextArea ? 'textarea' : 'input'"
      ref="inputRef"
      :class="
        clsx(
          'block w-full ml-auto text-sm font-mono',
          'border-transparent focus:outline-none focus:border-transparent focus:ring-0 focus:z-10',
          themeClasses(
            'text-black bg-white disabled:bg-neutral-100',
            'text-white bg-black disabled:bg-neutral-900',
          ),
          field.state.meta.errors.length > 0 && 'border-destructive-500 z-100',
          isTextArea ? 'min-h-[36px]' : 'h-lg',
          isTextArea &&
            isSecretField &&
            !secretShowing &&
            'secret-masked-textarea',
        )
      "
      :type="isTextArea ? null : getCurrentFormFieldType(fieldname)"
      :rows="isTextArea ? 4 : null"
      :value="field.state.value"
      tabindex="0"
      :placeholder="placeholder"
      data-lpignore="true"
      data-1p-ignore
      data-bwignore
      data-form-type="other"
      @keydown.tab="onTab"
      @input="
        (e: Event) => field.handleChange((e.target as HTMLInputElement).value)
      "
    />
    <Icon
      v-if="isSecretField"
      v-tooltip="secretShowing ? 'Hide Value' : 'Show Value'"
      :name="secretShowing ? 'hide' : 'eye'"
      size="xs"
      class="mr-xs cursor-pointer"
      @click="toggleSecretShowing"
    />
  </div>
</template>

<script setup lang="ts">
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, nextTick, ref } from "vue";

import { PropertyEditorPropWidgetKind } from "@/api/sdf/dal/property_editor";

const props = defineProps<{
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  field: any;
  fieldname: string;
  // widgetKind can be either a PropertyEditorPropWidgetKind object or a string (when coming from secret definitions)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  widgetKind?: PropertyEditorPropWidgetKind | string | any;
  placeholder?: null | string;
}>();

const isTextArea = computed(() => {
  // Description is always a textarea (special case - not from schema)
  if (props.fieldname === "Description") {
    return true;
  }

  // Check if the widgetKind indicates a textarea
  // The widgetKind can be either:
  //   - A string: "textArea", "password", "secret", "text", etc.
  //   - An object with the widget type as a key: { "textArea": {...} }
  //
  // Currently supported widget types for secret fields:
  //   - "textArea" -> renders as textarea (masked with CSS)
  //   - "password" -> renders as input type="password" (for future use)
  //   - "secret" or undefined -> defaults to password input
  //
  // To add support for other widget types (e.g., "codeEditor", "select"),
  // add the appropriate checks here and update the component rendering logic.

  if (props.widgetKind) {
    // Handle string widget kind
    if (
      typeof props.widgetKind === "string" &&
      props.widgetKind === "textArea"
    ) {
      return true;
    }
    // Handle object widget kind (legacy format)
    if (
      typeof props.widgetKind === "object" &&
      "textArea" in props.widgetKind
    ) {
      return true;
    }
  }

  return false;
});

const isSecretField = computed(() => {
  // Secret fields are all fields except Name and Description
  // Secret fields get masked inputs with show/hide toggle
  return props.fieldname !== "Name" && props.fieldname !== "Description";
});

const getFormFieldType = (fieldname: string) => {
  if (fieldname !== "Name" && fieldname !== "Description") {
    return "password";
  }
  return "text";
};

const secretShowing = ref(false);
const toggleSecretShowing = () => {
  secretShowing.value = !secretShowing.value;
};
const getCurrentFormFieldType = (fieldname: string) => {
  const type = getFormFieldType(fieldname);

  if (type === "password" && secretShowing.value) return "text";
  else return type;
};

const inputRef = ref<HTMLInputElement>();
const onTab = (e: KeyboardEvent) => {
  // This allows the user to Tab or Shift+Tab to go through the attribute fields
  const focusable = Array.from(
    document.querySelectorAll('[tabindex="0"]'),
  ) as HTMLElement[];
  const currentFocus = inputRef.value;
  if (!currentFocus) return;
  const index = focusable.indexOf(currentFocus);
  if (e.shiftKey) {
    e.preventDefault();
    nextTick(() => {
      if (currentFocus && focusable) {
        if (index > 0) {
          focusable[index - 1]?.focus();
        } else {
          focusable[focusable.length - 1]?.focus();
        }
      }
    });
  } else if (index === focusable.length - 1) {
    // When you hit the last attribute, go back to the
    // fuzzy search instead of searching the document for more things to tab to.
    e.preventDefault();
    nextTick(() => {
      focusable[0]?.focus();
    });
  }
};
</script>

<!-- CSS styles for secret-masked-textarea are in App.vue since they are used in multiple places -->
