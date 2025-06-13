<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center border',
        themeClasses(
          'text-black bg-white border-neutral-400',
          'text-white bg-black border-neutral-600',
        ),
      )
    "
  >
    <input
      :class="
        clsx(
          'block h-lg w-full ml-auto font-sm font-mono',
          'border-transparent focus:outline-none focus:border-transparent focus:ring-0 focus:z-100',
          themeClasses(
            'text-black bg-white disabled:bg-neutral-100',
            'text-white bg-black disabled:bg-neutral-900',
          ),
          field.state.meta.errors.length > 0 && 'border-destructive-500 z-100',
        )
      "
      :type="getCurrentFormFieldType(fieldname)"
      :value="field.state.value"
      tabindex="0"
      :placeholder="placeholder"
      data-1p-ignore
      @input="
      (e) =>
        field.handleChange((e.target as HTMLInputElement).value)
    "
    />
    <Icon
      v-if="getFormFieldType(fieldname) === 'password'"
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
import { ref } from "vue";

defineProps({
  field: { type: Object, required: true },
  fieldname: { type: String, required: true },
  placeholder: { type: String },
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
</script>
