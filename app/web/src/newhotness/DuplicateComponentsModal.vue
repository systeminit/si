<template>
  <ConfirmModal
    ref="modalRef"
    title="Duplicate"
    confirmLabel="Duplicate"
    size="xl"
    :loading="isConfirming"
    loadingText="Duplicating..."
    @confirm="emit('confirm', prefixName)"
  >
    <ErrorMessage v-if="requestError">{{ requestError }}</ErrorMessage>
    <div>
      Duplicated components keep the same name as the original. Add a prefix so
      you can easily distinguish them.
    </div>
    <div class="flex flex-row justify-between text-sm">
      <span class="mt-xs">Prefix</span>
      <div class="w-3/5 flex flex-col gap-2xs">
        <input
          v-model="prefixName"
          :class="
            clsx(
              'h-lg p-xs text-sm border font-mono cursor-text',
              'focus:outline-none focus:ring-0 focus:z-10',
              themeClasses(
                'text-shade-100 bg-white border-neutral-400 focus:border-action-500',
                'text-shade-0 bg-black border-neutral-600 focus:border-action-300',
              ),
            )
          "
          :placeholder="copyOf"
          @keydown.enter="confirm"
        />
        <span class="text-xs text-neutral-400">
          The name each duplicated component will begin with (E.g. Copy My
          Subnet)
        </span>
      </div>
    </div>
  </ConfirmModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { ErrorMessage, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ComponentId } from "@/api/sdf/dal/component";
import ConfirmModal from "./layout_components/ConfirmModal.vue";

const modalRef = ref<InstanceType<typeof ConfirmModal>>();
const requestError = ref<string | undefined>();
const isConfirming = ref(false);

// FORM FIELDS
const prefixName = ref<string>("");

const copyOf = ref<string>("Copy ");
// Static values
const viewIdRef = ref<string | undefined>();
const componentIdsRef = ref<ComponentId[] | undefined>();

function open(componentIds: ComponentId[], viewId: string) {
  componentIdsRef.value = componentIds;
  viewIdRef.value = viewId;

  requestError.value = undefined;
  prefixName.value = "";
  isConfirming.value = false;

  modalRef.value?.open();
}
function close() {
  modalRef.value?.close();
}

async function confirm() {
  if (isConfirming.value) return;

  const viewId = viewIdRef.value;
  const componentIds = componentIdsRef.value;
  if (!viewId || !componentIds) {
    return;
  }
  const nameToSend = prefixName.value || copyOf.value;
  isConfirming.value = true;
  emit("confirm", nameToSend);

  close();
}
const emit = defineEmits<{
  (e: "confirm", name: string): string;
}>();
defineExpose({ open, close });
</script>
