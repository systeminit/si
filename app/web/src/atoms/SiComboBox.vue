<template>
  <div class="mt-1 w-full relative">
    <Combobox v-model="inputValue">
      <ComboboxButton as="div">
        <ComboboxInput
          class="placeholder-neutral-400 border border-neutral-200 dark:border-neutral-600 text-sm rounded-sm shadow-sm w-full focus:border-action-300 pr-7"
          :class="clsx(themeClasses('bg-neutral-50', 'bg-neutral-900'))"
          @change="query = $event.target.value"
        />
        <Icon
          name="selector"
          class="absolute right-1.5 top-1.5 text-neutral-400"
        />
      </ComboboxButton>
      <ComboboxOptions
        class="absolute z-20 w-full mt-1 text-sm border dark:border-neutral-600 rounded-sm"
        :class="clsx(themeClasses('bg-neutral-50', 'bg-neutral-900'))"
        as="div"
      >
        <li
          class="flex flex-col gap-0.5 px-2.5 py-2.5 gap-1 border-b dark:border-neutral-600"
        >
          <div>
            <b>{{ filteredOptions.length }}</b> Result{{
              filteredOptions.length === 1 ? "" : "s"
            }}
          </div>
          <div class="text-neutral-500 italic text-smt">
            Type in the field above to filter the list below.
          </div>
        </li>
        <ul class="max-h-60 overflow-y-auto overflow-x-hidden">
          <ComboboxOption
            v-for="{ label, value } in filteredOptions"
            :key="value"
            v-slot="{ active, selected }"
            as="template"
            :value="value"
          >
            <li
              class="relative cursor-default select-none py-1.5 mx-2 dark:text-white rounded m-0.5 pl flex flex-row items-center"
              :class="{
                'bg-action-400 text-white': active,
                'text-gray-900': !active,
              }"
            >
              <Icon v-if="selected" name="check" class="mx-2" size="sm" />
              <span
                class="block truncate"
                :class="clsx(selected ? 'font-extrabold' : 'font-normal pl-9')"
              >
                {{ label }}
              </span>
            </li>
          </ComboboxOption>
        </ul>
      </ComboboxOptions>
    </Combobox>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import {
  Combobox,
  ComboboxInput,
  ComboboxOption,
  ComboboxOptions,
  ComboboxButton,
} from "@headlessui/vue";
import clsx from "clsx";
import { LabelList } from "@/api/sdf/dal/label_list";
import Icon from "@/ui-lib/icons/Icon.vue";
import { themeClasses } from "@/ui-lib/theme_tools";

const props = defineProps<{
  modelValue: string | number | undefined;
  options: LabelList<string | number>;
  title: string;
  id: string;
  description?: string;

  required?: boolean;
  alwaysValidate?: boolean;

  docLink?: string;

  disabled?: boolean;
}>();

const emit = defineEmits(["update:modelValue", "change"]);

const query = ref(props.modelValue === "string" ? props.modelValue : "");

const filteredOptions = computed(() =>
  query.value === ""
    ? props.options
    : props.options.filter(({ label }) => {
        return label.toLowerCase().includes(query.value.toLowerCase());
      }),
);

const inputValue = computed<string | number | undefined>({
  get() {
    return props.modelValue;
  },
  set(value) {
    emit("update:modelValue", value);
    emit("change", value);
    query.value = typeof value === "string" ? value : "";
  },
});
</script>
