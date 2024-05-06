<template>
  <VButton
    v-tooltip="'Attach Function'"
    tone="action"
    icon="link"
    iconRight="chevron--down"
    variant="ghost"
    :requestStatus="requestStatus"
    loadingText="Attaching new function..."
    size="2xs"
    @click="menuRef?.open"
  >
    <DropdownMenu ref="menuRef" compact>
      <DropdownMenuItem icon="plus" @select="emit('selectedAttachType', 'new')">
        New function
      </DropdownMenuItem>
      <DropdownMenuItem
        icon="func"
        @select="emit('selectedAttachType', 'existing')"
      >
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
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const emit = defineEmits<{
  (e: "selectedAttachType", type: "new" | "existing"): void;
}>();

const menuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>
