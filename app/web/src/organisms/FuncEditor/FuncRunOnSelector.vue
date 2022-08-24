<template>
  <div class="px-2 pb-5">
    <div class="flex items-center">
      <SelectMenu
        v-model="optionsState"
        class="w-4/5"
        :none-selected-label="noneSelectedLabel"
        :options="options"
        :disabled="disabled"
      />
      <SiButton
        label="Add"
        icon-style="alone"
        icon="plus"
        :disabled="disabled"
        @click="addOptions"
      />
    </div>
    <div>
      <h2>Selected components</h2>
      <h3 v-if="props.modelValue.length == 0">
        {{ noneSelectedBlurb }}
      </h3>
      <ul v-else>
        <li v-for="option in modelValue" :key="option.value">
          {{ option.label }}
          <SiButton
            label=""
            icon-style="alone"
            icon="cancel"
            :disabled="disabled"
            @click="removeOption(option)"
          />
        </li>
      </ul>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import SiButton from "@/atoms/SiButton.vue";

const props = defineProps<{
  options: Option[];
  modelValue: Option[];
  noneSelectedLabel: string;
  noneSelectedBlurb: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: Option[]): void;
  (e: "change", v: Option[]): void;
}>();

const optionsState = ref<Option[]>([]);

const addOptions = () => {
  const newOptions = Array.from(
    new Set(props.modelValue.concat(optionsState.value)),
  );

  emit("update:modelValue", newOptions);
  optionsState.value = [];
};

const removeOption = (remove: Option) => {
  const newOptions = props.modelValue.filter((opt) => opt !== remove);

  emit("update:modelValue", newOptions);
  emit("change", newOptions);
};
</script>
