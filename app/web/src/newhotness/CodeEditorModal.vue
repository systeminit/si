<template>
  <Modal ref="modalRef" :title="title" size="4xl" @close="updateValue">
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
      <CodeEditor
        :id="codeEditorId"
        v-model="newValueString"
        :recordId="codeEditorId"
      />
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, themeClasses, useModal } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ref } from "vue";
import CodeEditor from "@/components/CodeEditor.vue";

defineProps({
  title: { type: String, default: "Edit Value" },
  codeEditorId: { type: String, required: true },
});

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, isOpen } = useModal(modalRef);

const newValueString = ref<string>("");

const open = (startingValue = "") => {
  newValueString.value = startingValue;
  openModal();
};

const emit = defineEmits<{
  (e: "submit", value: string): void;
}>();

const updateValue = () => {
  emit("submit", newValueString.value);
};

defineExpose({
  open,
  isOpen,
});
</script>
