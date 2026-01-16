<template>
  <IconButton
    iconTone="action"
    icon="plus"
    size="sm"
    :requestStatus="requestStatus"
    :selected="menuRef?.isOpen"
    tooltip="Add Function"
    loadingTooltip="Adding function..."
    @click="onClick"
  >
    <DropdownMenu ref="menuRef" forceAlignRight>
      <DropdownMenuItem
        icon="plus"
        :disabled="assetStore.selectedSchemaVariant?.isLocked"
        @select="emit('selectedAttachType', 'new')"
      >
        New function
      </DropdownMenuItem>
      <DropdownMenuItem
        icon="link"
        :disabled="assetStore.selectedSchemaVariant?.isLocked"
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
import { DropdownMenu, DropdownMenuItem, IconButton } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";

const assetStore = useAssetStore();

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
