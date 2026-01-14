<template>
  <Modal
    ref="modalRef"
    title="Import an existing resource"
    buttonConfiguration="save"
    saveLabel="Import"
    :disableSave="formValue.trim() === ''"
    @save="submit"
  >
    <input
      ref="importInputRef"
      v-model="formValue"
      :class="
        clsx(
          'block w-full h-lg p-xs ml-auto text-sm border font-mono',
          themeClasses('text-shade-100 bg-shade-0 border-neutral-400', 'text-shade-0 bg-shade-100 border-neutral-600'),
        )
      "
      type="text"
      placeholder="Resource Id"
      @keydown.enter="submit"
    />
  </Modal>
</template>

<script setup lang="ts">
import { Modal, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ref } from "vue";

const modalRef = ref<InstanceType<typeof Modal>>();

const formValue = ref<string>("");

const open = (existingResourceId?: string) => {
  modalRef.value?.open();
  formValue.value = existingResourceId ?? "";
};

const close = () => {
  modalRef.value?.close();
};

const submit = () => {
  formValue.value = formValue.value.trim();

  if (formValue.value === "") {
    // must provide a resource id
    return;
  }

  emit("submit", formValue.value);
  close();
};

const emit = defineEmits<{
  (e: "submit", resourceId: string): void;
}>();

defineExpose({
  open,
  close,
});
</script>
