<template>
  <VButton
    v-tooltip="'Create Function'"
    tone="action"
    icon="plus"
    iconRight="chevron--down"
    variant="ghost"
    :requestStatus="requestStatus"
    loadingText="Creating new function..."
    size="2xs"
    @click="menuRef?.open"
  >
    <DropdownMenu ref="menuRef" compact>
      <DropdownMenuItem
        v-for="(fnLabel, fnKind) in CUSTOMIZABLE_FUNC_TYPES"
        :key="fnKind"
        icon="func"
        @select="emit('selectedFuncKind', fnKind)"
      >
        {{ fnLabel.singularLabel }}
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
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
} from "@/api/sdf/dal/func";

defineProps({
  label: { type: String, required: true },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
});

const emit = defineEmits<{
  (e: "selectedFuncKind", kind: CustomizableFuncKind): void;
}>();

const menuRef = ref<InstanceType<typeof DropdownMenu>>();
</script>
