import { computed, Ref } from "vue";
import Modal from "./Modal.vue";

export function useModal(
  modalRef: Ref<InstanceType<typeof Modal> | undefined>,
) {
  return {
    isOpen: computed(() => !!modalRef.value?.isOpen),
    open() {
      modalRef.value?.open();
    },
    close() {
      modalRef.value?.close();
    },
  };
}
