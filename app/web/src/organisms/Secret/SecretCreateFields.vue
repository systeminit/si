<template>
  <div v-for="field in secretKindFields.fields" :key="field.keyName">
    <div v-if="field.password" class="flex flex-row items-center w-full pb-2">
      <div
        class="w-1/2 pr-2 text-sm text-right text-gray-400 align-middle border-blue-100"
      >
        <label :for="idFor(field.keyName, field.password)"
          >{{ field.displayName }}:</label
        >
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          :id="idFor(field.keyName, field.password)"
          v-model="secretMessage[field.keyName]"
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
        <label :for="idFor(field.keyName, field.password)"
          >{{ field.displayName }}:</label
        >
      </div>
      <div class="w-1/2 align-middle">
        <SiTextBox
          :id="idFor(field.keyName, field.password)"
          v-model="secretMessage[field.keyName]"
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
import { SecretKindFields } from "@/api/sdf/dal/secret";

const emit = defineEmits(["update:modelValue"]);

const updateInput = () => {
  emit("update:modelValue", secretMessage);
};

const props = defineProps<{
  secretKindFields: SecretKindFields;
  modelValue: Record<string, string>;
}>();

const secretMessage = ref<Record<string, string>>(props.modelValue);

const idFor = (name: string, password: boolean): string => {
  if (password) {
    return "secret-password-" + name;
  }
  return "secret-text-" + name;
};
</script>
