<template>
  <IconButton
    iconTone="action"
    icon="link"
    :requestStatus="requestStatus"
    :selected="menuRef?.isOpen"
    tooltip="Attach Function"
    loadingTooltip="Attaching new function..."
    @click="onClick"
  >
    <DropdownMenu ref="menuRef" compact forceAlignRight>
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
  </IconButton>
</template>

<script setup lang="ts">
import { PropType, ref } from "vue";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import { DropdownMenu, DropdownMenuItem } from "@si/vue-lib/design-system";
import IconButton from "./IconButton.vue";

defineProps({
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const emit = defineEmits<{
  (e: "selectedAttachType", type: "new" | "existing"): void;
}>();

const menuRef = ref<InstanceType<typeof DropdownMenu>>();

const onClick = (e: MouseEvent) => {
  menuRef.value?.open(e);
};
</script>
