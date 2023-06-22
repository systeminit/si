<template>
  <VButton
    tone="action"
    icon="plus"
    icon-right="chevron--down"
    :variant="menuRef?.isOpen ? 'ghost' : 'solid'"
    :request-status="requestStatus"
    loading-text="Attaching new function..."
    size="sm"
    @click="menuRef?.open"
  >
    {{ label }}
    <DropdownMenu ref="menuRef">
      <DropdownMenuItem @select="emit('selectedAttachType', 'new')">
        New function
      </DropdownMenuItem>
      <DropdownMenuItem @select="emit('selectedAttachType', 'existing')">
        Existing
      </DropdownMenuItem>
    </DropdownMenu>
  </VButton>
</template>

<script setup lang="ts">
import { PropType, ref } from "vue";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import {
  DropdownMenu,
  DropdownMenuItem,
  VButton,
} from "@si/vue-lib/design-system";

defineProps({
  label: { type: String, required: true },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const emit = defineEmits<{
  (e: "selectedAttachType", type: "new" | "existing"): void;
}>();

const menuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>
