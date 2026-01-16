<template>
  <div>
    <div class="flex items-center">
      <SelectMenu
        v-model="optionsState"
        class="w-4/5"
        :noneSelectedLabel="`select ${thingLabel}...`"
        :options="options"
        :disabled="disabled"
      />
      <VButton
        label="Add"
        variant="ghost"
        tone="action"
        size="xs"
        icon="plus"
        :disabled="disabled || addIsDisabled"
        @click="addOptions"
      />
    </div>
    <div>
      <h2 class="py-xs text-sm">Selected {{ thingLabel }}:</h2>
      <p v-if="modelValue.length === 0" class="pl-sm text-xs italic">None selected. Select {{ thingLabel }} above...</p>
      <ul v-else class="list-disc list-inside flex flex-col">
        <li v-for="option in modelValue" :key="`${option.value}`" class="flex items-center text-sm pb-2 pl-4">
          <div class="pr-2" role="decoration">•</div>
          {{ option.label }}
          <div class="ml-auto">
            <VButton
              label=""
              icon="trash"
              tone="neutral"
              variant="transparent"
              :disabled="disabled"
              @click="removeOption(option)"
            />
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { nilId } from "@/utils/nilId";

const props = defineProps<{
  options: Option[];
  modelValue: Option[];
  thingLabel: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: Option[]): void;
  (e: "change", v: Option[]): void;
}>();

const noneVariant = { label: `select ${props.thingLabel}`, value: nilId() };
const optionsState = ref<Option>(noneVariant);

const addIsDisabled = computed(() => optionsState.value.value === nilId());

const addOptions = () => {
  const newOptions = Array.from(new Set(props.modelValue.concat(optionsState.value)));

  emit("update:modelValue", newOptions);
  optionsState.value = noneVariant;
  emit("change", newOptions);
};

const removeOption = (remove: Option) => {
  const newOptions = props.modelValue.filter((opt) => opt !== remove);

  emit("update:modelValue", newOptions);
  emit("change", newOptions);
};
</script>
