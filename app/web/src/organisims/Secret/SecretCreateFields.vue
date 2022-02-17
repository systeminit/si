<template>
  <div v-for="field in props.secretKind.fields" :key="field.name">
    <div v-if="field.password" class="flex flex-row items-center w-full pb-2">
      <div
        class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle border-blue-100"
      >
        <label :for="idFor(field.name, field.password)"
          >{{ field.displayName }}:</label
        >
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          :id="idFor(field.name, field.password)"
          v-model="createFields[field.name]"
          size="xs"
          placeholder=""
          :is-show-type="false"
          required
          type="password"
          @input="updateInput"
        />
      </div>
    </div>
    <div v-else class="flex flex-row items-center w-full pb-2">
      <div class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle">
        <label :for="idFor(field.name, field.password)"
          >{{ field.displayName }}:</label
        >
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          :id="idFor(field.name, field.password)"
          v-model="createFields[field.name]"
          size="xs"
          placeholder=""
          :is-show-type="false"
          required
          @input="updateInput"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { SecretKind } from "@/api/sdf/dal/secret";

const props = defineProps<{
  secretKind: SecretKind;
  modelValue: Record<string, string>;
}>();

const idFor = (name: string, password: boolean): string => {
  if (password) {
    return "secret-password-" + name;
  }
  return "secret-text-" + name;
};

const createFields = ref<Record<string, string>>({});

const emits = defineEmits(["update:modelValue"]);
const updateInput = () => {
  emits("update:modelValue", createFields);
};
</script>
