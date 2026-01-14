<template>
  <Modal ref="modalRef" :title="title" size="4xl" hideExitButton @close="cancel">
    <template #titleIcons>
      <button
        v-tooltip="'Cancel'"
        :class="clsx('modal-close-button', 'hover:scale-110 rounded-full opacity-80 hover:opacity-100 -mr-2 -my-2')"
        @click="cancel"
      >
        <Icon name="x" size="md" />
      </button>
    </template>
    <div
      :class="
        clsx(
          'relative h-[40vh]',
          '[&_.ͼ1.cm-editor.cm-focused]:outline-none [&_.ͼ1.cm-editor]:border',
          themeClasses(
            '[&_.ͼ1.cm-editor]:border-neutral-400 [&_.ͼ1.cm-editor.cm-focused]:border-action-500',
            '[&_.ͼ1.cm-editor]:border-neutral-600 [&_.ͼ1.cm-editor.cm-focused]:border-action-300',
          ),
          themeClasses('bg-shade-0', 'bg-shade-100'),
        )
      "
    >
      <CodeEditor :id="codeEditorId" v-model="newValueString" :recordId="codeEditorId" />
    </div>

    <div class="flex justify-end gap-sm mt-sm">
      <NewButton label="Cancel" @click="cancel" />
      <NewButton label="Save" tone="action" @click="save" />
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { Icon, Modal, NewButton, themeClasses, useModal } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { onBeforeUnmount, ref, watch } from "vue";
import CodeEditor from "@/components/CodeEditor.vue";

defineProps({
  title: { type: String, default: "Edit Value" },
  codeEditorId: { type: String, required: true },
});

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close: closeModal, isOpen } = useModal(modalRef);

const newValueString = ref<string>("");
const ignoreValueStringChanges = ref(true);

const open = (startingValue = "") => {
  newValueString.value = startingValue;
  submit.value = false;
  openModal();
};

const close = () => {
  ignoreValueStringChanges.value = true;
  closeModal();
};

const submit = ref(true);

const cancel = () => {
  emit("cancel");
  submit.value = false;
  close();
};

const save = () => {
  emit("submit", newValueString.value);
  close();
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === "Escape" && isOpen.value) {
    e.preventDefault();
    e.stopPropagation();
    cancel();
  }
};

watch(isOpen, (open) => {
  if (open) {
    document.addEventListener("keydown", handleKeydown, true);
  } else {
    document.removeEventListener("keydown", handleKeydown, true);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener("keydown", handleKeydown, true);
});

const emit = defineEmits<{
  (e: "submit", value: string): void;
  (e: "cancel"): void;
}>();

watch(
  () => newValueString.value,
  () => {
    // Handles the newValueString changing on modal open
    if (ignoreValueStringChanges.value) {
      ignoreValueStringChanges.value = false;
      return;
    }

    // If you change the string in the modal, then enable submit!
    submit.value = true;
  },
);

defineExpose({
  open,
  isOpen,
});
</script>
